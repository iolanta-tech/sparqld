---
"@context":
  schema: https://schema.org/
  title: schema:headline
  description: schema:description
"@id": reference/contexts/remote-contexts.md
"@type": schema:TechArticle
title: Remote contexts
description: Acquire a remote JSON-LD context with vitrify so sparqld can load sources that name its URL.
---

# :material-link-variant: Remote contexts

`sparqld` does not download context URLs (except the embedded
[default](default-context.md)). An absolute context URI fails until a local
registration supplies its body.

## :material-file-alert-outline: An unregistered URL fails

{{ example_data('recursive-context/unregistered/alpha-centauri.jsonld') }}

=== "Dependencies"

{{ source('docs/queries/contexts/unregistered-dependencies.rq', indent=4) }}

{{ stored_sparql('docs/queries/contexts/unregistered-dependencies.rq') | sparql_table | indent(4, true) }}

=== "Failed sources"

{{ source('docs/queries/contexts/unregistered-failures.rq', indent=4) }}

{{ stored_sparql('docs/queries/contexts/unregistered-failures.rq') | sparql_table | indent(4, true) }}

=== "Alpha Centauri"

{{ source('docs/queries/contexts/unregistered-alpha-centauri.rq', indent=4) }}

    ```text title="Expected result"
    {{ stored_sparql('docs/queries/contexts/unregistered-alpha-centauri.rq') }}
    ```

## :material-download-outline: Acquire the context with vitrify

[:fontawesome-brands-github: `iolanta-tech/vitrify`](https://github.com/iolanta-tech/vitrify)
writes the registration. Install and run
[`vitrify context`](https://vitrify.iolanta.tech/context/):

{{ command('cargo install vitrify') }}

{{ command('vitrify context https://www.w3.org/ns/solid/oidc-context.jsonld --as yamlld > contexts/oidc-context.yamlld') }}

Put that file under the served directory. The failure/success demo below uses a
tiny offline stand-in URL with the same registration shape:

{{ example_data('recursive-context/project/example.jsonld') }}

## :material-check-circle-outline: After registration

{{ example_data('recursive-context/project/alpha-centauri.jsonld') }}

=== "Dependencies"

{{ source('docs/queries/contexts/registered-dependencies.rq', indent=4) }}

{{ stored_sparql('docs/queries/contexts/registered-dependencies.rq') | sparql_table | indent(4, true) }}

=== "Declaration"

{{ source('docs/queries/contexts/registered-declaration.rq', indent=4) }}

    ```text title="Expected result"
    {{ stored_sparql('docs/queries/contexts/registered-declaration.rq') }}
    ```

=== "Failed sources"

{{ source('docs/queries/contexts/registered-failures.rq', indent=4) }}

{{ stored_sparql('docs/queries/contexts/registered-failures.rq') | sparql_table | indent(4, true) }}

=== "Alpha Centauri"

{{ source('docs/queries/contexts/registered-alpha-centauri.rq', indent=4) }}

    ```text title="Expected result"
    {{ stored_sparql('docs/queries/contexts/registered-alpha-centauri.rq') }}
    ```

Several nodes may share one body. Distinct bodies for one URI are a load error.
Changing a registered body reloads sources that requested it.

Authored contexts: [Project contexts](project-contexts.md).
