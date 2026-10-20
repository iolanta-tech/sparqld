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
    server::{MAX_QUERY_SIZE, evaluate_sparql, serialize_query_results},
    watcher::DirectoryWatcher,
};

pub(crate) fn serve(
    dataset: Store,
    directory: PathBuf,
    source_patterns: SourcePatterns,
    startup: crate::server::StartupGate,
) -> SparqldResult<()> {
    tokio::runtime::Runtime::new()?.block_on(async move {
        let server = McpServer {
            dataset: dataset.clone(),
            startup: startup.clone(),
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
struct McpServer {
    dataset: Store,
    startup: crate::server::StartupGate,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct QueryParameters {
    /// A read-only SPARQL query to run against the current dataset.
    query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct InvestigationParameters {
    /// A question to answer from the watched knowledge base.
    question: String,
}

#[tool_router]
impl McpServer {
    #[tool(
        description = "Run one read-only SPARQL query against the live file-backed dataset. SELECT and ASK return SPARQL Results JSON. CONSTRUCT and DESCRIBE return {triples: [{subject, predicate, object}]}, using SPARQL Results JSON RDF terms (type, value, optional datatype or xml:lang). Results are structured content with a matching JSON text fallback.",
        annotations(
            title = "Query sparqld",
            read_only_hint = true,
            open_world_hint = false
        )
    )]
    async fn query(&self, Parameters(parameters): Parameters<QueryParameters>) -> CallToolResult {
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

        let dataset = self.dataset.clone();
        let query = parameters.query;
        let response = tokio::task::spawn_blocking(move || query_dataset(&dataset, &query)).await;

        match response {
            Ok(Ok(output)) => CallToolResult::structured(output),
            Ok(Err(error)) => error_result(error),
            Err(error) => error_result(error),
        }
    }
}

#[prompt_router]
impl McpServer {
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

fn query_dataset(dataset: &Store, query: &str) -> Result<Value, String> {
    match evaluate_sparql(dataset, query).map_err(|error| error.to_string())? {
        QueryResults::Graph(triples) => {
            let triples = triples
                .map(|triple| {
                    let triple = triple.map_err(|error| error.to_string())?;
                    Ok(json!({
                        "subject": term_json(triple.subject.into()),
                        "predicate": term_json(triple.predicate.into()),
                        "object": term_json(triple.object),
                    }))
                })
                .collect::<Result<Vec<Value>, String>>()?;
            Ok(json!({"triples": triples}))
        }
        results => {
            let response = serialize_query_results(results).map_err(|error| error.to_string())?;
            serde_json::from_slice(&response.body).map_err(|error| error.to_string())
        }
    }
}

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

fn error_result(error: impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(error.to_string())])
}
