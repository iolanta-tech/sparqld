curl --fail --silent --show-error \
  -H 'Content-Type: application/sparql-query' \
  --data-binary @docs/examples/self-registering-context/queries/alice.rq \
  http://127.0.0.1:7878/
