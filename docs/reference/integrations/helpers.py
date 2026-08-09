from mkdocs_macros_sparqld import ensure_endpoint, run_query

endpoint = ensure_endpoint()
content_type, body = run_query('ASK { ?s ?p ?o }')
