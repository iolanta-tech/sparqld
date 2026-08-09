import sys

from rdflib import Graph
from SPARQLWrapper import JSON, TURTLE, SPARQLWrapper

client = SPARQLWrapper(sys.argv[1])

client.setQuery("""
SELECT ?name WHERE {
  <http://dbpedia.org/resource/Alpha_Centauri>
    <https://schema.org/name> ?name .
}
""")
client.setReturnFormat(JSON)
select_result = client.queryAndConvert()
print(f'SELECT: {select_result["results"]["bindings"][0]["name"]["value"]}')

client.setQuery('ASK { ?subject ?predicate ?object }')
client.setReturnFormat(JSON)
ask_result = client.queryAndConvert()
print(f'ASK: {str(ask_result["boolean"]).lower()}')

client.setQuery("""
CONSTRUCT {
  <http://dbpedia.org/resource/Alpha_Centauri>
    <https://schema.org/name> ?name .
}
WHERE {
  <http://dbpedia.org/resource/Alpha_Centauri>
    <https://schema.org/name> ?name .
}
""")
client.setReturnFormat(TURTLE)
construct_result = Graph().parse(
    data=client.queryAndConvert(),
    format='turtle',
)
name = next(construct_result.objects()).toPython()
print(f'CONSTRUCT: {name}')
