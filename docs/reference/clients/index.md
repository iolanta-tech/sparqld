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

-   <div class="client-heading" markdown>
    **[`sq`](https://github.com/ktk/sq)**
    <span class="client-recommended">Recommended</span>
    </div>

    ---

    ```console
    sq graphs
    ```

    Keeps endpoint URLs and prefixes in `.sq.toml`, making repeated queries
    concise.

-   ![Apache Jena logo](images/apache-jena.svg){ .client-logo }

    **[Apache Jena `rsparql`](https://jena.apache.org/documentation/query/sparql-remote.html)**

    ---

    ```console
    rsparql --service http://127.0.0.1:7737/ --query query.rq
    ```

    Sends a query file through Jena's SPARQL Protocol client.

-   ![Comunica logo](images/comunica.svg){ .client-logo }

    **[Comunica](https://comunica.dev/docs/query/getting_started/query_cli/)**

    ---

    ```console
    comunica-sparql sparql@http://127.0.0.1:7737/ -f query.rq
    ```

    The `sparql@` prefix identifies the source as an endpoint without requiring
    Service Description discovery.

-   ![RDFLib logo](images/rdflib.svg){ .client-logo }

    **[RDFLib `sparqlquery`](https://rdflib.readthedocs.io/en/stable/apidocs/rdflib.tools.sparqlquery/)**

    ---

    ```console
    sparqlquery http://127.0.0.1:7737/ --queryfile query.rq
    ```

    Queries the endpoint with RDFLib's Python command-line client.

</div>
