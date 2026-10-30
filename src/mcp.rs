//! Expose live read-only queries and investigation prompts through stdio MCP.

use std::path::PathBuf;

use oxigraph::{
    model::{Term, vocab::xsd},
    sparql::QueryResults,
    store::Store,
};
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, PromptMessage, Role},
    prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router,
    transport::stdio,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    SparqldResult,
    patterns::SourcePatterns,
    query::{QueryExecution, QueryPolicy},
    server::{
        MAX_QUERY_SIZE, QueryError, check_execution, classify_engine_error, evaluate_sparql,
        serialize_query_results,
    },
    watcher::DirectoryWatcher,
};

/// Serve stdio MCP requests while initializing and watching the selected directory.
pub(crate) fn serve(
    dataset: Store,
    directory: PathBuf,
    source_patterns: SourcePatterns,
    startup: crate::server::StartupGate,
    policy: QueryPolicy,
) -> SparqldResult<()> {
    tokio::runtime::Runtime::new()?.block_on(async move {
        let server = McpServer {
            dataset: dataset.clone(),
            startup: startup.clone(),
            policy,
        };
        let service = server.serve(stdio()).await?;

        let initialization = tokio::task::spawn_blocking(move || {
            DirectoryWatcher::start_with_patterns(directory, dataset, source_patterns)
        })
        .await;
        let _watcher = match initialization {
            Ok(Ok((watcher, stats))) => {
                log::info!("{}", crate::dataset_summary(stats));
                startup.mark_ready();
                Some(watcher)
            }
            Ok(Err(error)) => {
                log::error!("Could not initialize dataset: {error}");
                startup.mark_failed();
                None
            }
            Err(error) => {
                log::error!("Dataset initialization task failed: {error}");
                startup.mark_failed();
                None
            }
        };

        service.waiting().await?;
        Ok(())
    })
}

#[derive(Clone)]
/// MCP handlers sharing the live dataset, startup gate, and query policy.
struct McpServer {
    /// Live dataset queried by MCP requests.
    dataset: Store,
    /// Gate delaying queries until directory initialization finishes.
    startup: crate::server::StartupGate,
    /// Timeout policy applied to each incoming query.
    policy: QueryPolicy,
}

/// Cancel an engine query when its request handler is dropped.
struct CancelEngineOnDrop(oxigraph::sparql::CancellationToken);

impl Drop for CancelEngineOnDrop {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
/// Arguments accepted by the MCP query tool.
struct QueryParameters {
    /// A read-only SPARQL query to run against the current dataset.
    query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
/// Arguments accepted by the knowledge-base investigation prompt.
struct InvestigationParameters {
    /// A question to answer from the watched knowledge base.
    question: String,
}

#[tool_router]
impl McpServer {
    /// Wait for startup, evaluate a query, and propagate client cancellation to the engine.
    #[tool(
        description = "Run one read-only SPARQL query against the live file-backed dataset. SELECT and ASK return SPARQL Results JSON. CONSTRUCT and DESCRIBE return {triples: [{subject, predicate, object}]}, using SPARQL Results JSON RDF terms (type, value, optional datatype or xml:lang). Results are structured content with a matching JSON text fallback.",
        annotations(
            title = "Query sparqld",
            read_only_hint = true,
            open_world_hint = false
        )
    )]
    async fn query(
        &self,
        Parameters(parameters): Parameters<QueryParameters>,
        request_ct: tokio_util::sync::CancellationToken,
    ) -> CallToolResult {
        if parameters.query.len() > MAX_QUERY_SIZE {
            return error_result("Query is too large");
        }

        let startup = self.startup.clone();
        let readiness = tokio::task::spawn_blocking(move || startup.wait_until_ready()).await;
        match readiness {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return error_result(error),
            Err(error) => return error_result(error),
        }

        let execution = self.policy.begin();
        let engine_token = execution.token();
        let _cancel_on_drop = CancelEngineOnDrop(engine_token.clone());
        let dataset = self.dataset.clone();
        let query = parameters.query;
        let mut worker =
            tokio::task::spawn_blocking(move || query_dataset(&dataset, &query, &execution));

        tokio::select! {
            _ = request_ct.cancelled() => {
                engine_token.cancel();
                tool_result(worker.await)
            }
            response = &mut worker => tool_result(response),
        }
    }
}

#[prompt_router]
impl McpServer {
    /// Fill the bundled investigation prompt with the user's knowledge-base question.
    #[prompt(
        name = "investigate",
        title = "Investigate the knowledge base",
        description = "Answer a question by querying the watched knowledge base."
    )]
    async fn investigate(
        &self,
        Parameters(parameters): Parameters<InvestigationParameters>,
    ) -> Vec<PromptMessage> {
        vec![PromptMessage::new_text(
            Role::User,
            format!(
                include_str!("mcp/investigate.md"),
                question = parameters.question
            ),
        )]
    }
}

#[tool_handler(
    name = "sparqld",
    instructions = "Use query for read-only SPARQL over the watched directory. The default graph is the union of loaded source graphs. SPARQL Update is not supported."
)]
#[prompt_handler]
impl ServerHandler for McpServer {}

/// Evaluate a query and package its results as MCP structured content.
fn query_dataset(
    dataset: &Store,
    query: &str,
    execution: &QueryExecution,
) -> Result<CallToolResult, QueryError> {
    let output = match evaluate_sparql(dataset, query, execution)? {
        QueryResults::Graph(triples) => {
            let mut packed = Vec::new();
            for triple in triples {
                check_execution(execution)?;
                let triple = triple.map_err(|error| classify_engine_error(execution, error))?;
                packed.push(json!({
                    "subject": term_json(triple.subject.into()),
                    "predicate": term_json(triple.predicate.into()),
                    "object": term_json(triple.object),
                }));
            }
            json!({"triples": packed})
        }
        results => {
            let response = serialize_query_results(results, execution)?;
            serde_json::from_slice(&response.body)
                .map_err(|error| QueryError::Internal(error.to_string()))?
        }
    };
    check_execution(execution)?;
    Ok(CallToolResult::structured(output))
}

/// Convert a query worker's result or join failure into an MCP response.
fn tool_result(
    response: Result<Result<CallToolResult, QueryError>, tokio::task::JoinError>,
) -> CallToolResult {
    match response {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => error_result(error),
        Err(error) => error_result(error),
    }
}

/// Encode an RDF term using SPARQL Results JSON term conventions.
fn term_json(term: Term) -> Value {
    match term {
        Term::NamedNode(node) => json!({"type": "uri", "value": node.as_str()}),
        Term::BlankNode(node) => json!({"type": "bnode", "value": node.as_str()}),
        Term::Literal(literal) => {
            let mut value = json!({"type": "literal", "value": literal.value()});
            if let Some(language) = literal.language() {
                value["xml:lang"] = language.into();
            } else if literal.datatype() != xsd::STRING {
                value["datatype"] = literal.datatype().as_str().into();
            }
            value
        }
    }
}

/// Build an MCP error result from a displayable diagnostic.
fn error_result(error: impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(error.to_string())])
}
