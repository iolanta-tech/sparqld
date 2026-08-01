use std::io::{self, Read};
use std::mem;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use oxhttp::model::header::{ALLOW, CONTENT_TYPE};
use oxhttp::model::{Body, Method, Request, Response, StatusCode};
use oxhttp::{ListeningServer, Server};
use oxigraph::io::{RdfFormat, RdfSerializer};
use oxigraph::sparql::results::{QueryResultsFormat, QueryResultsSerializer};
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

const MAX_QUERY_SIZE: usize = 1024 * 1024;
const QUERY_RESULTS_MEDIA_TYPE: &str = "application/sparql-results+json";
const RDF_MEDIA_TYPE: &str = "text/turtle";

type HttpError = (StatusCode, String);
type HttpResult = Result<Response<Body>, HttpError>;

pub(crate) fn start(
    dataset: Arc<RwLock<Store>>,
    addresses: Vec<SocketAddr>,
) -> io::Result<ListeningServer> {
    let mut server = Server::new(move |request| handle_shared_request(request, &dataset))
        .with_server_name(concat!("sparqld/", env!("CARGO_PKG_VERSION")))
        .map_err(io::Error::other)?;

    for address in addresses {
        server = server.bind(address);
    }

    server.spawn()
}

fn handle_shared_request(request: &mut Request<Body>, dataset: &RwLock<Store>) -> Response<Body> {
    match dataset.read() {
        Ok(dataset) => handle_request(request, &dataset),
        Err(error) => text_response(StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
    }
}

fn handle_request(request: &mut Request<Body>, dataset: &Store) -> Response<Body> {
    try_handle_request(request, dataset)
        .unwrap_or_else(|(status, message)| text_response(status, message))
}

fn try_handle_request(request: &mut Request<Body>, dataset: &Store) -> HttpResult {
    if request.uri().path() != "/" {
        return Err((StatusCode::NOT_FOUND, "Not found".into()));
    }

    match *request.method() {
        Method::GET => match query_parameter(request.uri().query().unwrap_or_default())? {
            Some(query) => evaluate_query(dataset, &query),
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
                Some("application/sparql-query") => evaluate_query(dataset, &read_body(request)?),
                Some("application/x-www-form-urlencoded") => {
                    let body = read_body(request)?;
                    let query = query_parameter(&body)?.ok_or_else(|| {
                        (StatusCode::BAD_REQUEST, "Missing query parameter".into())
                    })?;
                    evaluate_query(dataset, &query)
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

fn evaluate_query(dataset: &Store, query: &str) -> HttpResult {
    let mut prepared = SparqlEvaluator::new()
        .parse_query(query)
        .map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))?;
    prepared.dataset_mut().set_default_graph_as_union();
    let results = prepared
        .on_store(dataset)
        .execute()
        .map_err(internal_server_error)?;

    match results {
        QueryResults::Solutions(solutions) => {
            let mut serializer = QueryResultsSerializer::from_format(QueryResultsFormat::Json)
                .serialize_solutions_to_writer(Vec::new(), solutions.variables().to_vec())
                .map_err(internal_server_error)?;
            for solution in solutions {
                serializer
                    .serialize(&solution.map_err(internal_server_error)?)
                    .map_err(internal_server_error)?;
            }
            binary_response(
                QUERY_RESULTS_MEDIA_TYPE,
                serializer.finish().map_err(internal_server_error)?,
            )
        }
        QueryResults::Boolean(result) => {
            let body = QueryResultsSerializer::from_format(QueryResultsFormat::Json)
                .serialize_boolean_to_writer(Vec::new(), result)
                .map_err(internal_server_error)?;
            binary_response(QUERY_RESULTS_MEDIA_TYPE, body)
        }
        QueryResults::Graph(triples) => {
            let mut serializer =
                RdfSerializer::from_format(RdfFormat::Turtle).for_writer(Vec::new());
            for triple in triples {
                serializer
                    .serialize_triple(&triple.map_err(internal_server_error)?)
                    .map_err(internal_server_error)?;
            }
            binary_response(
                RDF_MEDIA_TYPE,
                serializer.finish().map_err(internal_server_error)?,
            )
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
    use oxigraph::model::{NamedNodeRef, QuadRef};

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
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(content_type) = content_type {
            request = request.header(CONTENT_TYPE, content_type);
        }
        handle_request(&mut request.body(body.into()).unwrap(), dataset)
    }
}
