---
"@context":
  schema: https://schema.org/
  name: schema:name
  title: schema:headline
  description: schema:description
"@id": reference/clients/index.md
"@type": schema:TechArticle
hide: [toc]
name: Compatible SPARQL clients
title: Clients
description: Command-line SPARQL clients tested against sparqld.
---

# :material-console-network-outline: Clients

These command-line clients are exercised against the current `sparqld` binary
whenever this documentation is built.

{{ verify_clients() }}

<div class="grid cards client-gallery" markdown>

-   **[`sq`](sq/index.md)**
    <span class="client-recommended">Recommended</span>

    ---

    ```console
    sq -e http://127.0.0.1:7737/ graphs
    ```

    Use an endpoint URL directly, or [configure the endpoint and prefixes](sq/index.md)
    for concise repeated queries.

-   **[Apache Jena `rsparql`](https://jena.apache.org/documentation/query/sparql-remote.html)**

    ---

    ![Apache Jena logo](images/apache-jena.svg){ .client-logo }

    ```console
    rsparql --service http://127.0.0.1:7737/ --query query.rq
    ```

    Sends a query file through Jena's SPARQL Protocol client.

-   **[Comunica](https://comunica.dev/docs/query/getting_started/query_cli/)**

    ---

    ![Comunica logo](images/comunica.svg){ .client-logo }

    ```console
    comunica-sparql sparql@http://127.0.0.1:7737/ -f query.rq
    ```

    The `sparql@` prefix identifies the source as an endpoint without requiring
    Service Description discovery.

-   **[RDFLib `sparqlquery`](https://rdflib.readthedocs.io/en/stable/apidocs/rdflib.tools.sparqlquery/)**

    ---

    ![RDFLib logo](images/rdflib.svg){ .client-logo }

    ```console
    sparqlquery http://127.0.0.1:7737/ --queryfile query.rq
    ```

    Queries the endpoint with RDFLib's Python command-line client.

</div>
