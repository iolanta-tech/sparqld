curl --silent --show-error --fail-with-body \
  --header 'Content-Type: application/sparql-query' \
  --data-binary @docs/queries/context-dependencies.rq \
  http://127.0.0.1:7737/
