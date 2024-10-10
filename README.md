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
cargo run
```