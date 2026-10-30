//! Exercise live stdio MCP queries, reloads, and timeout recovery.

use std::{
    fs,
    future::Future,
    time::{Duration, Instant},
};

use rmcp::{
    ServiceExt,
    model::{CallToolRequest, CallToolRequestParams, ClientRequest, GetPromptRequestParams, Role},
    service::PeerRequestOptions,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::json;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn exposes_live_read_only_queries_over_stdio() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("data.ttl");
    write_turtle_source(&source, "first");
    let transport = TokioChildProcess::new(
        tokio::process::Command::new(env!("CARGO_BIN_EXE_sparqld")).configure(|command| {
            command.args(["mcp", directory.path().to_str().unwrap()]);
        }),
    )
    .unwrap();
    let mut client = complete_within(().serve(transport)).await.unwrap();

    let server = client.peer().peer_info().unwrap();
    assert_eq!(server.server_info.as_ref().unwrap().name, "sparqld");
    assert!(server.instructions.as_ref().unwrap().contains("read-only"));

    let tools = complete_within(client.list_tools(None))
        .await
        .unwrap()
        .tools;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "query");
    assert_eq!(
        tools[0]
            .annotations
            .as_ref()
            .and_then(|annotations| annotations.read_only_hint),
        Some(true)
    );

    let prompts = complete_within(client.list_prompts(None))
        .await
        .unwrap()
        .prompts;
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "investigate");
    assert_eq!(prompts[0].arguments.as_ref().unwrap()[0].name, "question");
    assert_eq!(
        prompts[0].arguments.as_ref().unwrap()[0].required,
        Some(true)
    );

    let prompt = complete_within(
        client.get_prompt(
            GetPromptRequestParams::new("investigate").with_arguments(
                json!({"question": "What is in this dataset?"})
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        ),
    )
    .await
    .unwrap();
    assert_eq!(prompt.messages[0].role, Role::User);
    let prompt_text = prompt.messages[0].content.as_text().unwrap().text.as_str();
    assert!(prompt_text.contains("First discover"));
    assert!(prompt_text.contains("What is in this dataset?"));

    let select = query(
        &client,
        "SELECT ?value WHERE { <urn:subject> <urn:predicate> ?value }",
    )
    .await;
    assert_eq!(
        select.structured_content.as_ref().unwrap(),
        &serde_json::from_str::<serde_json::Value>(include_str!("../docs/results/mcp-select.json"))
            .unwrap()
    );
    assert_json_text_matches_structure(&select);

    let ask = query(&client, "ASK { <urn:subject> <urn:predicate> ?value }").await;
    assert_eq!(
        ask.structured_content.as_ref().unwrap(),
        &serde_json::from_str::<serde_json::Value>(include_str!("../docs/results/mcp-ask.json"))
            .unwrap()
    );
    assert_json_text_matches_structure(&ask);

    let absent = query(&client, "ASK { <urn:absent> ?predicate ?value }").await;
    assert_eq!(
        absent.structured_content.as_ref().unwrap(),
        &json!({"head": {}, "boolean": false})
    );
    assert_json_text_matches_structure(&absent);

    let construct = query(
        &client,
        "CONSTRUCT { <urn:subject> <urn:predicate> ?value } WHERE { <urn:subject> <urn:predicate> ?value }",
    )
    .await;
    assert_eq!(
        construct.structured_content.as_ref().unwrap(),
        &serde_json::from_str::<serde_json::Value>(include_str!("../docs/results/mcp-graph.json"))
            .unwrap()
    );
    assert_json_text_matches_structure(&construct);

    let describe = query(&client, "DESCRIBE <urn:subject>").await;
    assert_eq!(describe.structured_content, construct.structured_content);
    assert_json_text_matches_structure(&describe);

    let empty = query(&client, "CONSTRUCT { ?s ?p ?o } WHERE { FILTER(false) }").await;
    assert_eq!(empty.structured_content, Some(json!({"triples": []})));
    assert_json_text_matches_structure(&empty);

    let mixed = query(
        &client,
        r#"
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
        CONSTRUCT {
            _:node <urn:iri> <urn:object> ;
                   <urn:language> "Bonjour"@fr ;
                   <urn:typed> "42"^^xsd:integer ;
                   <urn:escaped> "a\"b\\c\n" .
            <urn:subject> <urn:blank> _:node .
        } WHERE {}
    "#,
    )
    .await;
    let triples = mixed.structured_content.as_ref().unwrap()["triples"]
        .as_array()
        .unwrap();
    assert_eq!(triples.len(), 5);
    let object_for = |predicate: &str| {
        &triples
            .iter()
            .find(|triple| triple["predicate"]["value"] == predicate)
            .unwrap()["object"]
    };
    assert_eq!(
        object_for("urn:iri"),
        &json!({"type": "uri", "value": "urn:object"})
    );
    assert_eq!(
        object_for("urn:language"),
        &json!({"type": "literal", "value": "Bonjour", "xml:lang": "fr"})
    );
    assert_eq!(
        object_for("urn:typed"),
        &json!({"type": "literal", "value": "42", "datatype": "http://www.w3.org/2001/XMLSchema#integer"})
    );
    assert_eq!(
        object_for("urn:escaped"),
        &json!({"type": "literal", "value": "a\"b\\c\n"})
    );
    let blank = object_for("urn:blank");
    assert_eq!(blank["type"], "bnode");
    assert!(!blank["value"].as_str().unwrap().starts_with("_:"));
    assert!(
        triples
            .iter()
            .filter(|triple| triple["predicate"]["value"] != "urn:blank")
            .all(|triple| triple["subject"] == *blank)
    );
    assert_json_text_matches_structure(&mixed);

    let empty_describe = query(&client, "DESCRIBE <urn:absent>").await;
    assert_eq!(
        empty_describe.structured_content,
        Some(json!({"triples": []}))
    );

    let update = query(&client, "INSERT DATA { <urn:s> <urn:p> <urn:o> }").await;
    assert_eq!(update.is_error, Some(true));

    write_turtle_source(&source, "second");
    let deadline = Instant::now() + REQUEST_TIMEOUT;
    loop {
        let response = query(
            &client,
            "SELECT ?value WHERE { <urn:subject> <urn:predicate> ?value }",
        )
        .await;
        if response.structured_content.as_ref().unwrap()["results"]["bindings"][0]["value"]["value"]
            == "second"
        {
            break;
        }
        assert!(Instant::now() < deadline, "the MCP dataset did not reload");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    assert!(
        complete_within(client.close_with_timeout(REQUEST_TIMEOUT))
            .await
            .unwrap()
            .is_some()
    );
}

fn assert_json_text_matches_structure(result: &rmcp::model::CallToolResult) {
    let text = &result.content[0].as_text().unwrap().text;
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(&parsed, result.structured_content.as_ref().unwrap());
}

async fn query(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    query: &str,
) -> rmcp::model::CallToolResult {
    complete_within(
        client.call_tool(
            CallToolRequestParams::new("query")
                .with_arguments(json!({"query": query}).as_object().unwrap().clone()),
        ),
    )
    .await
    .unwrap()
}

async fn complete_within<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(REQUEST_TIMEOUT, future)
        .await
        .expect("MCP server did not respond within 5 seconds")
}

fn write_turtle_source(path: &std::path::Path, value: &str) {
    fs::write(
        path,
        format!("@prefix example: <urn:> .\nexample:subject example:predicate \"{value}\" .\n"),
    )
    .unwrap();
}

const SLOW_QUERY_SELECT: &str =
    "SELECT * WHERE { ?a ?p ?c . ?d ?p ?f . ?g ?p ?i . ?j ?p ?l . ?m ?p ?o }";
const SLOW_QUERY_ASK: &str = "ASK { { SELECT (COUNT(*) AS ?c) WHERE { ?a ?p ?c . ?d ?p ?f . ?g ?p ?i . ?j ?p ?l . ?m ?p ?o } } }";
const SLOW_QUERY_CONSTRUCT: &str =
    "CONSTRUCT { ?a ?p ?c } WHERE { ?a ?p ?c . ?d ?p ?f . ?g ?p ?i . ?j ?p ?l . ?m ?p ?o }";

#[tokio::test]
async fn times_out_slow_queries_and_recovers() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("data.ttl");
    write_cross_product_source(&source, 60);
    let transport = TokioChildProcess::new(
        tokio::process::Command::new(env!("CARGO_BIN_EXE_sparqld")).configure(|command| {
            command.args([
                "mcp",
                "--query-timeout",
                "1",
                directory.path().to_str().unwrap(),
            ]);
        }),
    )
    .unwrap();
    let mut client = complete_within(().serve(transport)).await.unwrap();

    for query_text in [SLOW_QUERY_SELECT, SLOW_QUERY_ASK, SLOW_QUERY_CONSTRUCT] {
        let response = query(&client, query_text).await;
        assert_eq!(response.is_error, Some(true), "{query_text}");
        assert_eq!(
            response.content[0].as_text().unwrap().text,
            "Query exceeded the 1 s timeout"
        );
    }

    let recovered = query(&client, "ASK {}").await;
    assert_ne!(recovered.is_error, Some(true));
    assert_eq!(
        recovered.structured_content.as_ref().unwrap(),
        &json!({"head": {}, "boolean": true})
    );

    let mut reloaded = fs::read_to_string(&source).unwrap();
    reloaded.push_str("example:reloaded example:p \"yes\" .\n");
    fs::write(&source, reloaded).unwrap();
    let deadline = Instant::now() + REQUEST_TIMEOUT;
    loop {
        let response = query(&client, "ASK { <urn:reloaded> <urn:p> \"yes\" }").await;
        if response.structured_content.as_ref().unwrap()["boolean"] == true {
            break;
        }
        assert!(Instant::now() < deadline, "the MCP dataset did not reload");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let handle = client
        .peer()
        .send_cancellable_request(
            ClientRequest::CallToolRequest(CallToolRequest::new(
                CallToolRequestParams::new("query").with_arguments(
                    json!({"query": SLOW_QUERY_SELECT})
                        .as_object()
                        .unwrap()
                        .clone(),
                ),
            )),
            PeerRequestOptions::no_options(),
        )
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;
    handle.cancel(Some("test".into())).await.unwrap();

    let after_cancel = complete_within(async { query(&client, "ASK {}").await }).await;
    assert_ne!(after_cancel.is_error, Some(true));

    assert!(
        complete_within(client.close_with_timeout(REQUEST_TIMEOUT))
            .await
            .unwrap()
            .is_some()
    );
}

fn write_cross_product_source(path: &std::path::Path, triples: usize) {
    let mut body = String::from("@prefix example: <urn:> .\n");
    for index in 0..triples {
        body.push_str(&format!("example:n{index} example:p example:n{index} .\n"));
    }
    fs::write(path, body).unwrap();
}
