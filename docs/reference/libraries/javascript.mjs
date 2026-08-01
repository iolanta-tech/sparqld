import { QueryEngine } from '@comunica/query-sparql';


const source = { type: 'sparql', value: process.argv[2] };
const client = new QueryEngine();

const bindings = await client.queryBindings(`
  SELECT ?name WHERE {
    <http://dbpedia.org/resource/Alpha_Centauri>
      <https://schema.org/name> ?name .
  }
`, { sources: [source] });
const rows = await bindings.toArray();
console.log(`SELECT: ${rows[0].get('name').value}`);

const ask = await client.queryBoolean(
  'ASK { ?subject ?predicate ?object }',
  { sources: [source] },
);
console.log(`ASK: ${ask}`);

const quads = await client.queryQuads(`
  CONSTRUCT {
    <http://dbpedia.org/resource/Alpha_Centauri>
      <https://schema.org/name> ?name .
  }
  WHERE {
    <http://dbpedia.org/resource/Alpha_Centauri>
      <https://schema.org/name> ?name .
  }
`, { sources: [source] });
const graph = await quads.toArray();
console.log(`CONSTRUCT: ${graph[0].object.value}`);
