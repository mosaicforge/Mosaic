# README ai-search

## API
ai-search is a REST API that can be queried at the adress 127.0.0.0:3000.
The only available route is /question that takes a JSON containing the question for the knowledge graph.


## Start command
```
cargo run --bin ai-search -- --neo4j-uri neo4j://localhost:7687     --neo4j-user neo4j     --neo4j-pass neo4j     --gemini-api-key <Your gemini api key>

```