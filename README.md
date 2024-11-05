## Setup
### 1. Start Neo4j
```bash
docker run \
    --publish=7474:7474 --publish=7687:7687 \
    --volume=data:/data \
    --env=NEO4J_AUTH=none \
    neo4j
```

### 2. Download root dump
```bash
wget https://gateway.lighthouse.storage/ipfs/bafkreif4acly7y46hx7optzfxtehxotizgqjz5h5vszo7vtmzsnm4ktxjy
```

## Run
```bash
cargo run --bin main -- \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j 
```

Codegen
```bash
cargo run --package kg-cli -- \
    --neo4j-uri neo4j://localhost:7687 \
    --neo4j-user neo4j \
    --neo4j-pass neo4j \
    codegen
```

## Docker compose
### 1. Start neo4j
```bash
cd docker/
docker compose up neo4j
```

### 2. Start kg-node
```bash
cd docker/
docker compose up kg-node
```