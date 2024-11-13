use std::collections::HashMap;

use anyhow::Error;
use chrono::{DateTime, Utc};
use clap::{Args, Parser};
use futures::{stream, StreamExt, TryStreamExt};
use ipfs::{deserialize, IpfsClient};
use kg_core::{
    models::{EditProposal, Space, SpaceType},
    pb::{
        geo::{self, GeoOutput},
        grc20,
    },
};
use kg_node::kg::id::{create_geo_id, create_space_id};
use kg_node::web3_utils::checksum_address;
use kg_node::{kg, network_ids};
use prost::Message;
use substreams_sink_rust::pb::sf::substreams::rpc::v2::BlockScopedData;
use substreams_sink_rust::Sink;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const ENDPOINT_URL: &str = "https://geotest.substreams.pinax.network:443";
const PKG_FILE: &str = "geo-substream.spkg";
const MODULE_NAME: &str = "geo_out";

const START_BLOCK: i64 = 25327;
const STOP_BLOCK: u64 = 0;

#[tokio::main]
async fn main() -> Result<(), Error> {
    set_log_level();
    init_tracing();
    let args = AppArgs::parse();

    let kg_client = kg::Client::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    if args.reset_db {
        kg_client.reset_db(args.rollup).await?;
    };

    let sink = KgSink::new(kg_client);

    sink.run(ENDPOINT_URL, PKG_FILE, MODULE_NAME, START_BLOCK, STOP_BLOCK)
        .await?;

    Ok(())
}

struct BlockMetadata {
    cursor: String,
    block_number: u64,
    timestamp: DateTime<Utc>,
    request_id: String,
}

impl BlockMetadata {
    pub fn from_substreams_block(block: &BlockScopedData) -> Self {
        let clock = block.clock.as_ref().unwrap();
        let timestamp = DateTime::from_timestamp(
            clock.timestamp.as_ref().unwrap().seconds,
            clock.timestamp.as_ref().unwrap().nanos as u32,
        )
        .expect("received timestamp should always be valid");

        Self {
            cursor: block.cursor.clone(),
            block_number: clock.number,
            timestamp,
            request_id: create_geo_id(),
        }
    }
}

struct KgSink {
    ipfs: IpfsClient,
    kg: kg::Client,
}

impl KgSink {
    pub fn new(kg: kg::Client) -> Self {
        Self {
            ipfs: IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/"),
            kg,
        }
    }

    /// Handles `GeoSpaceCreated` events. `ProposalProcessed` events are used to determine
    /// the space's ID in cases where the space is imported.
    ///
    /// The method returns the IDs of the spaces that were successfully created.
    pub async fn handle_spaces_created(
        &self,
        spaces_created: &[geo::GeoSpaceCreated],
        proposals_processed: &[geo::ProposalProcessed],
        block: &BlockMetadata,
    ) -> Result<Vec<String>, Error> {
        // Match the space creation events with their corresponding initial proposal (if any)
        let initial_proposals = spaces_created
            .iter()
            .filter_map(|event| {
                proposals_processed
                    .iter()
                    .find(|proposal| {
                        checksum_address(&proposal.plugin_address, None)
                            == checksum_address(&event.space_address, None)
                    })
                    .map(|proposal| (event.space_address.clone(), proposal))
            })
            .collect::<HashMap<_, _>>();

        // For spaces with an initial proposal, get the space ID from the import (if available)
        let space_ids = stream::iter(initial_proposals)
            .filter_map(|(space_address, proposal_processed)| async move {
                let ipfs_hash = proposal_processed.content_uri.replace("ipfs://", "");
                self.ipfs
                    .get::<grc20::Import>(&ipfs_hash, true)
                    .await
                    .ok()
                    .map(|import| {
                        (
                            space_address,
                            create_space_id(
                                &import.previous_network,
                                &import.previous_contract_address,
                            ),
                        )
                    })
            })
            .collect::<HashMap<_, _>>()
            .await;

        // Create the spaces
        let created_ids: Vec<_> = stream::iter(spaces_created)
            .then(|event| async {
                let space_id = space_ids
                    .get(&event.space_address)
                    .cloned()
                    .unwrap_or(create_space_id(network_ids::GEO, &event.dao_address));

                tracing::info!(
                    "Block #{} ({}): Creating space {}",
                    block.block_number,
                    block.timestamp,
                    space_id
                );

                self.kg
                    .create_space(Space {
                        id: space_id.to_string(),
                        network: network_ids::GEO.to_string(),
                        contract_address: event.space_address.to_string(),
                        dao_contract_address: event.dao_address.to_string(),
                        r#type: SpaceType::Public,
                        created_at: block.timestamp,
                        created_at_block: block.block_number,
                    })
                    .await?;

                anyhow::Ok(space_id)
            })
            .try_collect()
            .await?;

        Ok(created_ids)
    }

    pub async fn handle_proposals_processed(
        &self,
        proposals_processed: &[geo::ProposalProcessed],
        created_space_ids: &[String],
        block: &BlockMetadata,
    ) -> Result<(), Error> {
        let proposals = stream::iter(proposals_processed)
            .then(|proposal| async {
                let edits = self.fetch_edit_proposals(proposal).await?;
                anyhow::Ok(edits)
            })
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // TODO: Create "synthetic" proposals for newly created spaces and
        // personal spaces

        stream::iter(proposals)
            .map(Ok) // Need to wrap the proposal in a Result to use try_for_each
            .try_for_each(|proposal| async {
                tracing::info!(
                    "Block #{} ({}): Creating edit proposal {}",
                    block.block_number,
                    block.timestamp,
                    proposal.proposal_id
                );
                self.kg.process_edit(proposal).await
            })
            .await?;

        Ok(())
    }

    async fn fetch_edit_proposals(
        &self,
        proposal_processed: &geo::ProposalProcessed,
    ) -> Result<Vec<EditProposal>, Error> {
        let space = if let Some(space) = self
            .kg
            .get_space_by_address(&proposal_processed.plugin_address)
            .await?
        {
            space
        } else {
            tracing::warn!(
                "Matching space in Proposal not found for plugin address {}",
                proposal_processed.plugin_address
            );
            return Ok(vec![]);
        };

        let bytes = self
            .ipfs
            .get_bytes(&proposal_processed.content_uri.replace("ipfs://", ""), true)
            .await?;

        let metadata = deserialize::<grc20::Metadata>(&bytes)?;
        match metadata.r#type() {
            grc20::ActionType::AddEdit => {
                let edit = deserialize::<grc20::Edit>(&bytes)?;
                Ok(vec![EditProposal {
                    name: edit.name,
                    proposal_id: edit.id,
                    space: space.id,
                    space_address: space.contract_address,
                    creator: edit.authors[0].clone(),
                    ops: edit.ops,
                }])
            }
            grc20::ActionType::ImportSpace => {
                let import = deserialize::<grc20::Import>(&bytes)?;
                let edits = stream::iter(import.edits)
                    .map(|edit| async move {
                        let hash = edit.replace("ipfs://", "");
                        self.ipfs.get::<grc20::ImportEdit>(&hash, true).await
                    })
                    .buffer_unordered(10)
                    .try_collect::<Vec<_>>()
                    .await?;

                Ok(edits
                    .into_iter()
                    .map(|edit| EditProposal {
                        name: edit.name,
                        proposal_id: edit.id,
                        space: space.id.clone(),
                        space_address: space.contract_address.clone(),
                        creator: edit.authors[0].clone(),
                        ops: edit.ops,
                    })
                    .collect())
            }
            _ => Err(anyhow::anyhow!("Invalid metadata")),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
struct SinkError(#[from] Box<dyn std::error::Error + Send + Sync>);

impl substreams_sink_rust::Sink for KgSink {
    type Error = SinkError;

    async fn process_block_scoped_data(&self, data: &BlockScopedData) -> Result<(), Self::Error> {
        let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();

        let block = BlockMetadata::from_substreams_block(data);

        let value =
            GeoOutput::decode(output.value.as_slice()).map_err(|e| SinkError(Box::new(e)))?;

        // Handle new space creation
        let created_space_ids = self
            .handle_spaces_created(&value.spaces_created, &value.proposals_processed, &block)
            .await
            .map_err(|e| SinkError(format!("{e:?}").into()))?;

        // Handle proposal processing
        self.handle_proposals_processed(&value.proposals_processed, &created_space_ids, &block)
            .await
            .map_err(|e| SinkError(format!("{e:?}").into()))?;

        Ok(())
    }
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,

    /// Whether or not to roll up the relations
    #[arg(long, default_value_t = true)]
    rollup: bool,

    /// Whether or not to reset the database
    #[arg(long)]
    reset_db: bool,
}

#[derive(Debug, Args)]
struct Neo4jArgs {
    /// Neo4j database host
    #[arg(long)]
    neo4j_uri: String,

    /// Neo4j database user name
    #[arg(long)]
    neo4j_user: String,

    /// Neo4j database user password
    #[arg(long)]
    neo4j_pass: String,
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "stdout=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn set_log_level() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
}
