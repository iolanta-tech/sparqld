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

{{ example_data('recursive-context/project/alpha-centauri.jsonld') }}

=== "Dependencies"

{{ source('docs/examples/recursive-context/results/unregistered/dependencies/results.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ source('docs/examples/recursive-context/results/unregistered/failures/results.json', indent=4, title='Expected result') }}

=== "Alpha Centauri"

{{ source('docs/examples/recursive-context/results/unregistered/alpha-centauri/results.json', indent=4, title='Expected result') }}

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

=== "Dependencies"

{{ source('docs/queries/context-dependencies.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/dependencies/results.json', indent=4, title='Expected result') }}

=== "Declaration"

{{ example_data('recursive-context/queries/declaration.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/declaration/results.json', indent=4, title='Expected result') }}

=== "Failed sources"

{{ example_data('recursive-context/queries/failures.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/failures/results.json', indent=4, title='Expected result') }}

=== "Alpha Centauri"

{{ example_data('recursive-context/queries/alpha-centauri.rq', indent=4) }}

{{ source('docs/examples/recursive-context/results/initial/alpha-centauri/results.json', indent=4, title='Expected result') }}

Several nodes may share one body. Distinct bodies for one URI are a load error.
Changing a registered body reloads sources that requested it.

Authored contexts: [Project contexts](project-contexts.md).
