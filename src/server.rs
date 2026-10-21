use std::io::{self, Read};
use std::mem;
use std::net::SocketAddr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use oxhttp::model::header::{ALLOW, CONTENT_TYPE};
use oxhttp::model::{Body, Method, Request, Response, StatusCode};
use oxhttp::{ListeningServer, Server};
use oxigraph::io::{RdfFormat, RdfSerializer};
use oxigraph::sparql::results::{QueryResultsFormat, QueryResultsSerializer};
use oxigraph::sparql::{QueryEvaluationError, QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

use crate::query::{QueryExecution, QueryPolicy};

pub(crate) const MAX_QUERY_SIZE: usize = 1024 * 1024;
pub(crate) const QUERY_RESULTS_MEDIA_TYPE: &str = "application/sparql-results+json";
const RDF_MEDIA_TYPE: &str = "text/turtle";

type HttpError = (StatusCode, String);
type HttpResult = Result<Response<Body>, HttpError>;

#[derive(Debug)]
pub(crate) struct QueryResponse {
    pub(crate) content_type: &'static str,
    pub(crate) body: Vec<u8>,
}

pub(crate) enum QueryError {
    BadRequest(String),
    Internal(String),
    Timeout(Duration),
    Cancelled,
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest(message) | Self::Internal(message) => formatter.write_str(message),
            Self::Timeout(timeout) => formatter.write_str(&timeout_message(*timeout)),
            Self::Cancelled => formatter.write_str("Query cancelled"),
        }
    }
}

impl From<crate::query::Interruption> for QueryError {
    fn from(interruption: crate::query::Interruption) -> Self {
        match interruption {
            crate::query::Interruption::TimedOut(timeout) => Self::Timeout(timeout),
            crate::query::Interruption::Cancelled => Self::Cancelled,
        }
    }
}

pub(crate) fn timeout_message(timeout: Duration) -> String {
    format!("Query exceeded the {} s timeout", timeout.as_secs())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StartupState {
    Loading,
    Ready,
    Failed,
}

#[derive(Clone)]
pub(crate) struct StartupGate {
    state: Arc<(Mutex<StartupState>, Condvar)>,
}

impl StartupGate {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new((Mutex::new(StartupState::Loading), Condvar::new())),
        }
    }

    pub(crate) fn mark_ready(&self) {
        self.set(StartupState::Ready);
    }

    pub(crate) fn mark_failed(&self) {
        self.set(StartupState::Failed);
    }

    pub(crate) fn wait_until_ready(&self) -> Result<(), String> {
        match self.wait()? {
            StartupState::Ready => Ok(()),
            StartupState::Failed => Err("Initial dataset failed to load".into()),
            StartupState::Loading => unreachable!("startup gate returned while loading"),
        }
    }

    fn set(&self, state: StartupState) {
        let (current, changed) = &*self.state;
        *current.lock().expect("startup state lock poisoned") = state;
        changed.notify_all();
    }

    fn wait(&self) -> Result<StartupState, String> {
        let (state, changed) = &*self.state;
        let mut state = state.lock().map_err(|error| error.to_string())?;
        while *state == StartupState::Loading {
            state = changed.wait(state).map_err(|error| error.to_string())?;
        }
        Ok(*state)
    }
}

pub(crate) fn start(
    dataset: Store,
    startup: StartupGate,
    policy: QueryPolicy,
    addresses: Vec<SocketAddr>,
) -> io::Result<ListeningServer> {
    let mut server =
        Server::new(move |request| handle_shared_request(request, &dataset, &startup, &policy))
            .with_server_name(concat!("sparqld/", env!("CARGO_PKG_VERSION")))
            .map_err(io::Error::other)?;

    for address in addresses {
        server = server.bind(address);
    }

    server.spawn()
}

fn handle_shared_request(
    request: &mut Request<Body>,
    dataset: &Store,
    startup: &StartupGate,
    policy: &QueryPolicy,
) -> Response<Body> {
    if request.uri().path() != "/" {
        return text_response(StatusCode::NOT_FOUND, "Not found");
    }

    match startup.wait() {
        Ok(StartupState::Ready) => {}
        Ok(StartupState::Failed) => {
            return text_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Initial dataset failed to load",
            );
        }
        Ok(StartupState::Loading) => unreachable!("startup gate returned while loading"),
        Err(error) => return text_response(StatusCode::INTERNAL_SERVER_ERROR, error),
    }

    handle_request(request, dataset, policy)
}

fn handle_request(
    request: &mut Request<Body>,
    dataset: &Store,
    policy: &QueryPolicy,
) -> Response<Body> {
    try_handle_request(request, dataset, policy)
        .unwrap_or_else(|(status, message)| text_response(status, message))
}

fn try_handle_request(
    request: &mut Request<Body>,
    dataset: &Store,
    policy: &QueryPolicy,
) -> HttpResult {
    if request.uri().path() != "/" {
        return Err((StatusCode::NOT_FOUND, "Not found".into()));
    }

    match *request.method() {
        Method::GET => match query_parameter(request.uri().query().unwrap_or_default())? {
            Some(query) => evaluate_query(dataset, &query, policy),
            None if request.uri().query().is_none() => Ok(text_response(
                StatusCode::OK,
                "sparqld read-only SPARQL endpoint",
            )),
            None => Err((StatusCode::BAD_REQUEST, "Missing query parameter".into())),
        },
        Method::POST => {
            let content_type = request
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .map(|value| value.split(';').next().unwrap_or_default().trim());

            match content_type {
                Some("application/sparql-query") => {
                    evaluate_query(dataset, &read_body(request)?, policy)
                }
                Some("application/x-www-form-urlencoded") => {
                    let body = read_body(request)?;
                    let query = query_parameter(&body)?.ok_or_else(|| {
                        (StatusCode::BAD_REQUEST, "Missing query parameter".into())
                    })?;
                    evaluate_query(dataset, &query, policy)
                }
                Some("application/sparql-update") => Err(read_only_error()),
                Some(content_type) => Err((
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    format!("Unsupported Content-Type: {content_type}"),
                )),
                None => Err((StatusCode::BAD_REQUEST, "Missing Content-Type".into())),
            }
        }
        _ => Err(read_only_error()),
    }
}

fn query_parameter(encoded: &str) -> Result<Option<String>, HttpError> {
    let mut queries = form_urlencoded::parse(encoded.as_bytes())
        .filter(|(name, _)| name == "query")
        .map(|(_, value)| value.into_owned());
    let query = queries.next();
    if queries.next().is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "The query parameter must not be repeated".into(),
        ));
    }
    Ok(query)
}

fn read_body(request: &mut Request<Body>) -> Result<String, HttpError> {
    if request
        .body()
        .len()
        .is_some_and(|length| length > MAX_QUERY_SIZE as u64)
    {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "Query is too large".into()));
    }

    let mut bytes = Vec::new();
    mem::take(request.body_mut())
        .take((MAX_QUERY_SIZE + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
    if bytes.len() > MAX_QUERY_SIZE {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "Query is too large".into()));
    }
    String::from_utf8(bytes).map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))
}

fn evaluate_query(dataset: &Store, query: &str, policy: &QueryPolicy) -> HttpResult {
    let execution = policy.begin();
    execute_query(dataset, query, &execution)
        .map_err(|error| match error {
            QueryError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            QueryError::Internal(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            QueryError::Timeout(timeout) => {
                (StatusCode::SERVICE_UNAVAILABLE, timeout_message(timeout))
            }
            QueryError::Cancelled => (StatusCode::SERVICE_UNAVAILABLE, "Query cancelled".into()),
        })
        .and_then(|response| binary_response(response.content_type, response.body))
}

pub(crate) fn execute_query(
    dataset: &Store,
    query: &str,
    execution: &QueryExecution,
) -> Result<QueryResponse, QueryError> {
    serialize_query_results(evaluate_sparql(dataset, query, execution)?, execution)
}

pub(crate) fn evaluate_sparql<'a>(
    dataset: &'a Store,
    query: &str,
    execution: &QueryExecution,
) -> Result<QueryResults<'a>, QueryError> {
    let mut prepared = SparqlEvaluator::new()
        .with_cancellation_token(execution.token())
        .parse_query(query)
        .map_err(|error| QueryError::BadRequest(error.to_string()))?;
    prepared.dataset_mut().set_default_graph_as_union();
    prepared
        .on_store(dataset)
        .execute()
        .map_err(|error| classify_engine_error(execution, error))
}

/// Maps an engine error to a timeout or cancellation when the query was
/// interrupted, and otherwise reports an internal failure.
pub(crate) fn classify_engine_error(
    execution: &QueryExecution,
    error: QueryEvaluationError,
) -> QueryError {
    if matches!(error, QueryEvaluationError::Cancelled) {
        return execution
            .check()
            .map_or(QueryError::Cancelled, QueryError::from);
    }
    QueryError::Internal(error.to_string())
}

/// Reports whether the deadline has passed or the query was cancelled.
pub(crate) fn check_execution(execution: &QueryExecution) -> Result<(), QueryError> {
    match execution.check() {
        Some(interruption) => Err(interruption.into()),
        None => Ok(()),
    }
}

pub(crate) fn serialize_query_results(
    results: QueryResults<'_>,
    execution: &QueryExecution,
) -> Result<QueryResponse, QueryError> {
    match results {
        QueryResults::Solutions(solutions) => {
            let mut serializer = QueryResultsSerializer::from_format(QueryResultsFormat::Json)
                .serialize_solutions_to_writer(Vec::new(), solutions.variables().to_vec())
                .map_err(|error| QueryError::Internal(error.to_string()))?;
            for solution in solutions {
                check_execution(execution)?;
                serializer
                    .serialize(&solution.map_err(|error| classify_engine_error(execution, error))?)
                    .map_err(|error| QueryError::Internal(error.to_string()))?;
            }
            let body = serializer
                .finish()
                .map_err(|error| QueryError::Internal(error.to_string()))?;
            check_execution(execution)?;
            Ok(QueryResponse {
                content_type: QUERY_RESULTS_MEDIA_TYPE,
                body,
            })
        }
        QueryResults::Boolean(result) => {
            let body = QueryResultsSerializer::from_format(QueryResultsFormat::Json)
                .serialize_boolean_to_writer(Vec::new(), result)
                .map_err(|error| QueryError::Internal(error.to_string()))?;
            check_execution(execution)?;
            Ok(QueryResponse {
                content_type: QUERY_RESULTS_MEDIA_TYPE,
                body,
            })
        }
        QueryResults::Graph(triples) => {
            let mut serializer =
                RdfSerializer::from_format(RdfFormat::Turtle).for_writer(Vec::new());
            for triple in triples {
                check_execution(execution)?;
                serializer
                    .serialize_triple(
                        &triple.map_err(|error| classify_engine_error(execution, error))?,
                    )
                    .map_err(|error| QueryError::Internal(error.to_string()))?;
            }
            let body = serializer
                .finish()
                .map_err(|error| QueryError::Internal(error.to_string()))?;
            check_execution(execution)?;
            Ok(QueryResponse {
                content_type: RDF_MEDIA_TYPE,
                body,
            })
        }
    }
}

fn binary_response(content_type: &'static str, body: Vec<u8>) -> HttpResult {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .body(body.into())
        .map_err(internal_server_error)
}

fn text_response(status: StatusCode, body: impl Into<String>) -> Response<Body> {
    let mut response = Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8");
    if status == StatusCode::METHOD_NOT_ALLOWED {
        response = response.header(ALLOW, "GET, POST");
    }
    response.body(body.into().into()).unwrap()
}

fn read_only_error() -> HttpError {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        "This SPARQL endpoint is read-only".into(),
    )
}

fn internal_server_error(error: impl std::fmt::Display) -> HttpError {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxigraph::model::{GraphNameRef, NamedNode, NamedNodeRef, QuadRef};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn serves_a_landing_response() {
        let response = request(Method::GET, "/", None, "");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "sparqld read-only SPARQL endpoint"
        );
    }

    #[test]
    fn evaluates_a_get_query() {
        let response = request(
            Method::GET,
            "/?query=ASK%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D",
            None,
            "",
        );

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            QUERY_RESULTS_MEDIA_TYPE
        );
        assert!(
            response
                .into_body()
                .to_string()
                .unwrap()
                .contains("\"boolean\":false")
        );
    }

    #[test]
    fn queries_the_union_of_named_graphs_by_default() {
        let dataset = Store::new().unwrap();
        let subject = NamedNodeRef::new("urn:subject").unwrap();
        let predicate = NamedNodeRef::new("urn:predicate").unwrap();
        let object = NamedNodeRef::new("urn:object").unwrap();
        let graph = NamedNodeRef::new("urn:graph").unwrap();
        dataset
            .insert(QuadRef::new(subject, predicate, object, graph))
            .unwrap();

        let response = request_on(
            &dataset,
            Method::GET,
            "/?query=ASK%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D",
            None,
            "",
        );

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .into_body()
                .to_string()
                .unwrap()
                .contains("\"boolean\":true")
        );
    }

    #[test]
    fn evaluates_a_post_query() {
        let response = request(
            Method::POST,
            "/",
            Some("application/sparql-query"),
            "SELECT * WHERE { ?s ?p ?o }",
        );

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            QUERY_RESULTS_MEDIA_TYPE
        );
        assert!(
            response
                .into_body()
                .to_string()
                .unwrap()
                .contains("\"bindings\":[]")
        );
    }

    #[test]
    fn evaluates_a_form_query() {
        let response = request(
            Method::POST,
            "/",
            Some("application/x-www-form-urlencoded"),
            "query=ASK%20%7B%20%7D",
        );

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn serializes_graph_queries_as_turtle() {
        let response = request(
            Method::POST,
            "/",
            Some("application/sparql-query"),
            "CONSTRUCT WHERE { ?s ?p ?o }",
        );

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            RDF_MEDIA_TYPE
        );
    }

    #[test]
    fn accepts_content_type_parameters() {
        let response = request(
            Method::POST,
            "/",
            Some("application/sparql-query; charset=utf-8"),
            "ASK {}",
        );

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn rejects_a_missing_query_parameter() {
        let response = request(Method::GET, "/?format=json", None, "");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "Missing query parameter"
        );
    }

    #[test]
    fn rejects_a_repeated_query_parameter() {
        let response = request(
            Method::GET,
            "/?query=ASK%20%7B%7D&query=ASK%20%7B%7D",
            None,
            "",
        );

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "The query parameter must not be repeated"
        );
    }

    #[test]
    fn rejects_a_post_without_a_content_type() {
        let response = request(Method::POST, "/", None, "ASK {}");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "Missing Content-Type"
        );
    }

    #[test]
    fn rejects_an_unsupported_content_type() {
        let response = request(Method::POST, "/", Some("application/json"), "{}");

        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "Unsupported Content-Type: application/json"
        );
    }

    #[test]
    fn rejects_an_oversized_query() {
        let response = request(
            Method::POST,
            "/",
            Some("application/sparql-query"),
            "x".repeat(MAX_QUERY_SIZE + 1),
        );

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "Query is too large"
        );
    }

    #[test]
    fn rejects_updates() {
        let response = request(
            Method::POST,
            "/",
            Some("application/sparql-update"),
            "INSERT DATA { <urn:s> <urn:p> <urn:o> }",
        );

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers().get(ALLOW).unwrap(), "GET, POST");
    }

    #[test]
    fn rejects_unknown_paths() {
        let response = request(Method::GET, "/update", None, "");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn rejects_other_http_methods() {
        let response = request(Method::DELETE, "/", None, "");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers().get(ALLOW).unwrap(), "GET, POST");
    }

    #[test]
    fn waits_for_root_requests_until_startup_is_ready() {
        let dataset = Store::new().unwrap();
        let startup = StartupGate::new();
        let (started_sender, started) = mpsc::channel();

        let get_dataset = dataset.clone();
        let get_startup = startup.clone();
        let get_started = started_sender.clone();
        let get = thread::spawn(move || {
            get_started.send(()).unwrap();
            let mut request = Request::builder()
                .method(Method::GET)
                .uri("/?query=ASK%20%7B%20%3Fs%20%3Fp%20%3Fo%20%7D")
                .body(Body::default())
                .unwrap();
            let response = handle_shared_request(
                &mut request,
                &get_dataset,
                &get_startup,
                &QueryPolicy::new(None),
            );
            (
                response.status().as_u16(),
                response.into_body().to_string().unwrap(),
            )
        });

        let post_dataset = dataset.clone();
        let post_startup = startup.clone();
        let post = thread::spawn(move || {
            started_sender.send(()).unwrap();
            let mut request = Request::builder()
                .method(Method::POST)
                .uri("/")
                .header(CONTENT_TYPE, "application/sparql-query")
                .body(Body::from("ASK { ?s ?p ?o }"))
                .unwrap();
            let response = handle_shared_request(
                &mut request,
                &post_dataset,
                &post_startup,
                &QueryPolicy::new(None),
            );
            (
                response.status().as_u16(),
                response.into_body().to_string().unwrap(),
            )
        });

        started.recv().unwrap();
        started.recv().unwrap();
        thread::sleep(Duration::from_millis(20));
        assert!(!get.is_finished());
        assert!(!post.is_finished());

        startup.mark_ready();

        for (status, body) in [get.join().unwrap(), post.join().unwrap()] {
            assert_eq!(status, StatusCode::OK.as_u16());
            assert!(body.contains("\"boolean\":false"));
        }
    }

    #[test]
    fn rejects_root_requests_after_startup_fails() {
        let dataset = Store::new().unwrap();
        let startup = StartupGate::new();
        startup.mark_failed();
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Body::default())
            .unwrap();

        let response =
            handle_shared_request(&mut request, &dataset, &startup, &QueryPolicy::new(None));

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "Initial dataset failed to load"
        );
    }

    #[test]
    fn rejects_unknown_paths_while_starting() {
        let dataset = Store::new().unwrap();
        let startup = StartupGate::new();
        let mut request = Request::builder()
            .method(Method::GET)
            .uri("/readyz")
            .body(Body::default())
            .unwrap();

        let response =
            handle_shared_request(&mut request, &dataset, &startup, &QueryPolicy::new(None));

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    fn request<B: Into<Body>>(
        method: Method,
        uri: &str,
        content_type: Option<&str>,
        body: B,
    ) -> Response<Body> {
        request_on(&Store::new().unwrap(), method, uri, content_type, body)
    }

    fn request_on<B: Into<Body>>(
        dataset: &Store,
        method: Method,
        uri: &str,
        content_type: Option<&str>,
        body: B,
    ) -> Response<Body> {
        request_with_policy(
            dataset,
            &QueryPolicy::new(None),
            method,
            uri,
            content_type,
            body,
        )
    }

    fn request_with_policy<B: Into<Body>>(
        dataset: &Store,
        policy: &QueryPolicy,
        method: Method,
        uri: &str,
        content_type: Option<&str>,
        body: B,
    ) -> Response<Body> {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        handle_request(&mut request.body(body.into()).unwrap(), dataset, policy)
    }

    /// Builds a store whose every triple matches an unbound triple pattern, so
    /// an n-way self-join scans a combinatorial number of candidates.
    fn cross_product_store(triples: usize) -> Store {
        let dataset = Store::new().unwrap();
        for index in 0..triples {
            let subject = NamedNode::new(format!("urn:node:{index}")).unwrap();
            dataset
                .insert(QuadRef::new(
                    &subject,
                    NamedNodeRef::new("urn:predicate").unwrap(),
                    &subject,
                    GraphNameRef::DefaultGraph,
                ))
                .unwrap();
        }
        dataset
    }

    #[test]
    fn times_out_a_long_running_select_query() {
        let dataset = cross_product_store(60);
        let policy = QueryPolicy::new(Some(Duration::from_secs(1)));

        let response = request_with_policy(
            &dataset,
            &policy,
            Method::POST,
            "/",
            Some("application/sparql-query"),
            "SELECT * WHERE { ?a ?b ?c . ?d ?e ?f . ?g ?h ?i . ?j ?k ?l . ?m ?n ?o }",
        );

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            response.into_body().to_string().unwrap(),
            "Query exceeded the 1 s timeout"
        );
    }

    #[test]
    fn times_out_a_long_running_ask_query() {
        let execution = QueryExecution::expired(Duration::from_secs(20));
        let error = serialize_query_results(QueryResults::Boolean(true), &execution).unwrap_err();

        assert!(matches!(error, QueryError::Timeout(timeout) if timeout.as_secs() == 20));
    }

    #[test]
    fn times_out_a_long_running_graph_query() {
        let dataset = cross_product_store(8);
        let execution = QueryExecution::expired(Duration::from_secs(20));

        let error = execute_query(
            &dataset,
            "CONSTRUCT { ?a ?b ?c } WHERE { ?a ?b ?c . ?d ?e ?f }",
            &execution,
        )
        .unwrap_err();

        assert!(matches!(error, QueryError::Timeout(timeout) if timeout.as_secs() == 20));
    }

    #[test]
    fn times_out_a_select_query_with_an_expired_deadline() {
        let dataset = cross_product_store(8);
        let execution = QueryExecution::expired(Duration::from_secs(20));

        let error = execute_query(&dataset, "SELECT * WHERE { ?s ?p ?o }", &execution).unwrap_err();

        assert!(matches!(error, QueryError::Timeout(timeout) if timeout.as_secs() == 20));
    }

    #[test]
    fn serves_a_successful_query_after_a_timeout() {
        let dataset = cross_product_store(60);
        let policy = QueryPolicy::new(Some(Duration::from_secs(1)));

        let timed_out = request_with_policy(
            &dataset,
            &policy,
            Method::POST,
            "/",
            Some("application/sparql-query"),
            "SELECT * WHERE { ?a ?b ?c . ?d ?e ?f . ?g ?h ?i . ?j ?k ?l . ?m ?n ?o }",
        );
        assert_eq!(timed_out.status(), StatusCode::SERVICE_UNAVAILABLE);

        let succeeded = request_with_policy(
            &dataset,
            &policy,
            Method::GET,
            "/?query=ASK%20%7B%7D",
            None,
            "",
        );

        assert_eq!(succeeded.status(), StatusCode::OK);
        assert!(
            succeeded
                .into_body()
                .to_string()
                .unwrap()
                .contains("\"boolean\":true")
        );
    }
}
