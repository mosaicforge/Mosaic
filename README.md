<h1 align="center">
 Mosaic
</h1>
<p align="center">
  <img width="400" alt="mosaic" src="https://pbs.twimg.com/profile_images/1890229652326612992/SsN44tyU_400x400.jpg"/>
</p>
<p align="center">
Orchestration layer for Rig-based agents

### With `docker-compose`
```bash
cd docker/
docker compose up
```

### Without docker
### 1. Start Neo4j
```bash
docker run \
    --publish=7474:7474 --publish=7687:7687 \
    --volume=./data:/data \
    --env=NEO4J_AUTH=none \
    neo4j
```

### 2. Compile and run the indexer
In a separate terminal, run the following commands:
```bash
CFLAGS='-std=gnu17' cargo run --release --bin sink -- \
    --no-versioning \
    --no-governance \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j
```

```bash
CFLAGS='-std=gnu17' cargo run --bin api -- \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j
```
Schema introspection

```
npx get-graphql-schema+alpha http://127.0.0.1:8080/graphql > api/schema.graphql
```

## MCP Server
```bash
CFLAGS='-std=gnu17' cargo run --bin mcp-server -- \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j
```

### Local testing with sample data
Start the neo4j database and run the following command:
```bash
CFLAGS='-std=gnu17' cargo run --example seed_data
```

The IDs of the sample data can be found in `sink/examples/seed_data.rs`.

## GRC20 CLI
Coming soon™️
