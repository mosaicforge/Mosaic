use chrono::DateTime;
use futures::{stream, StreamExt, TryStreamExt};
use ipfs::IpfsClient;
use kg_core::{ids::create_geo_id, models::BlockMetadata, pb::geo::GeoOutput};
use prost::Message;
use substreams_sink_rust::pb::sf::substreams::rpc::v2::BlockScopedData;

use crate::kg::{self, client::DatabaseError};

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("IPFS error: {0}")]
    IpfsError(#[from] ipfs::Error),

    #[error("prost error: {0}")]
    Prost(#[from] prost::DecodeError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    // #[error("KG error: {0}")]
    // KgError(#[from] kg::Error),
    #[error("Error processing event: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct EventHandler {
    pub(crate) ipfs: IpfsClient,
    pub(crate) kg: kg::Client,
}

impl EventHandler {
    pub fn new(kg: kg::Client) -> Self {
        Self {
            ipfs: IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/"),
            kg,
        }
    }
}

fn get_block_metadata(block: &BlockScopedData) -> anyhow::Result<BlockMetadata> {
    let clock = block.clock.as_ref().unwrap();
    let timestamp = DateTime::from_timestamp(
        clock.timestamp.as_ref().unwrap().seconds,
        clock.timestamp.as_ref().unwrap().nanos as u32,
    )
    .ok_or(anyhow::anyhow!("get_block_metadata: Invalid timestamp"))?;

    Ok(BlockMetadata {
        cursor: block.cursor.clone(),
        block_number: clock.number,
        timestamp,
        request_id: create_geo_id(),
    })
}

impl substreams_sink_rust::Sink for EventHandler {
    type Error = HandlerError;

    async fn process_block_scoped_data(&self, data: &BlockScopedData) -> Result<(), Self::Error> {
        let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();

        let block =
            get_block_metadata(data).map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        let value = GeoOutput::decode(output.value.as_slice())?;

        // Handle new space creation
        let created_space_ids = self
            .handle_spaces_created(&value.spaces_created, &value.proposals_processed, &block)
            .await?;

        // Handle personal space creation
        stream::iter(&value.personal_plugins_created)
            .map(Ok)
            .try_for_each(|event| async { self.handle_personal_space_created(event, &block).await })
            .await?;

        // Handle new governance plugin creation
        stream::iter(&value.governance_plugins_created)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_governance_plugin_created(event, &block).await
            })
            .await?;

        // Handle subspaces creation
        stream::iter(&value.subspaces_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_subspace_added(event, &block).await })
            .await?;

        // Handle subspace removal
        stream::iter(&value.subspaces_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_subspace_removed(event, &block).await })
            .await?;

        // Handle initial editors added
        stream::iter(&value.initial_editors_added)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_initial_space_editors_added(event, &block).await
            })
            .await?;

        // Handle proposal creation
        stream::iter(&value.proposals_created)
            .map(Ok)
            .try_for_each(|event| async { self.handle_proposal_created(event, &block).await })
            .await?;

        // Handle proposal processing
        self.handle_proposals_processed(&value.proposals_processed, &created_space_ids, &block)
            .await?;

        // Handle members added
        stream::iter(&value.members_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_member_added(event, &block).await })
            .await?;

        // Handle members removed
        stream::iter(&value.members_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_member_removed(event, &block).await })
            .await?;

        // Handle editors added
        stream::iter(&value.editors_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_editor_added(event, &block).await })
            .await?;

        // Handle editors removed
        stream::iter(&value.editors_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_editor_removed(event, &block).await })
            .await?;

        // Handle vote cast
        stream::iter(&value.votes_cast)
            .map(Ok)
            .try_for_each(|event| async { self.handle_vote_cast(event, &block).await })
            .await?;

        // Handle executed proposal
        stream::iter(&value.executed_proposals)
            .map(Ok)
            .try_for_each(|event| async { self.handle_proposal_executed(event, &block).await })
            .await?;

        Ok(())
    }
}
