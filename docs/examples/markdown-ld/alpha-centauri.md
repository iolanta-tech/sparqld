---
"@context":
  - https://json-ld.org/contexts/dollar-convenience.jsonld
  - schema: https://schema.org/
    dbo: http://dbpedia.org/ontology/
    dbp: http://dbpedia.org/property/
    dbr: http://dbpedia.org/resource/
    $: schema:name
    constellation:
      "@id": dbo:constellation
      "@type": "@id"
    contains:
      "@id": schema:hasPart
      "@type": "@id"
    is-orbited-by:
      "@reverse": dbp:star
$id: dbr:Alpha_Centauri
$: Alpha Centauri
schema:description: The closest star system to the Solar System.
constellation:
  $id: dbr:Centaurus
  $: Centaurus
contains:
  - $: Alpha Centauri AB
    contains:
      - $type: dbo:Star
        $: Alpha Centauri A
      - $type: dbo:Star
        $: Alpha Centauri B
    is-orbited-by:
      - $id: dbr:Proxima_Centauri
        $type: dbo:Star
        $: Proxima Centauri
        is-orbited-by:
          - $id: dbr:Proxima_Centauri_b
            $type: dbo:Planet
            $: Proxima Centauri b
          - $id: dbr:Proxima_Centauri_d
            $type: dbo:Planet
            $: Proxima Centauri d
---

# Alpha Centauri

Alpha Centauri is the closest star system to the Solar System.
