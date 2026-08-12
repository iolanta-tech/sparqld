import { QueryEngine } from '@comunica/query-sparql';


const source = { type: 'sparql', value: `http://127.0.0.1:${process.env.PORT}/` };
const client = new QueryEngine();
const bindings = await client.queryBindings(
  `SELECT DISTINCT ?name WHERE {
    <http://dbpedia.org/resource/Alpha_Centauri>
      <https://schema.org/name> ?name .
  }`,
  { sources: [source] },
);
const rows = await bindings.toArray();
const result = rows.map(row => Object.fromEntries(
  [...row].map(([variable, term]) => [variable.value, term.value]),
));

console.log(JSON.stringify(result, null, 2));
