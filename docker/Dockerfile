FROM rust:1.81.0 AS builder

WORKDIR /kg-node
COPY . .
RUN apt-get update && apt-get upgrade -y
RUN apt-get install libssl-dev protobuf-compiler -y
RUN cargo build --release --bin kg-node

FROM debian:bookworm-slim

ENV neo4j_uri ""
ENV neo4j_user ""
ENV neo4j_pass ""
ENV SUBSTREAMS_API_TOKEN ""
ENV SUBSTREAMS_ENDPOINT_URL ""

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y libssl-dev
COPY --from=builder /kg-node/target/release/kg-node .
COPY --from=builder /kg-node/geo-substream.spkg .
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

CMD ./kg-node \
    --reset-db \
    --rollup \
    --neo4j-uri $neo4j_uri \
    --neo4j-user $neo4j_user \
    --neo4j-pass $neo4j_pass
