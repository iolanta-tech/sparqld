import json
import os

from SPARQLWrapper import JSON, SPARQLWrapper

client = SPARQLWrapper(f'http://127.0.0.1:{os.environ["PORT"]}/')
client.setQuery(
    """SELECT DISTINCT ?name WHERE {
  <http://dbpedia.org/resource/Alpha_Centauri>
    <https://schema.org/name> ?name .
}"""
)
client.setReturnFormat(JSON)
print(json.dumps(client.queryAndConvert(), indent=2))
