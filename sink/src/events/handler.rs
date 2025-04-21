use chrono::DateTime;
use futures::{stream, StreamExt, TryStreamExt};
use grc20_core::{
    block::BlockMetadata, error::DatabaseError, ids::create_geo_id, indexer_ids, mapping::Query,
    neo4rs, pb::geo::GeoOutput,
};
use ipfs::IpfsClient;
use prost::Message;
use substreams_utils::pb::sf::substreams::rpc::v2::BlockScopedData;

use crate::{blacklist, metrics};
use cache::{CacheConfig, KgCache};
use std::sync::Arc;
use std::time::Duration;

#[derive(thiserror::Error, Debug)]
pub enum HandlerError {
    #[error("IPFS error: {0}")]
    IpfsError(#[from] ipfs::Error),

    #[error("prost error: {0}")]
    Prost(#[from] prost::DecodeError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error("Cache error: {0}")]
    CacheError(#[from] cache::CacheError),

    // #[error("KG error: {0}")]
    // KgError(#[from] kg::Error),
    #[error("Error processing event: {0}")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub struct EventHandler {
    pub(crate) ipfs: IpfsClient,
    pub(crate) neo4j: neo4rs::Graph,
    pub(crate) cache: Arc<KgCache>,
    pub(crate) spaces_blacklist: Vec<String>,
}

impl EventHandler {
    pub fn new(neo4j: neo4rs::Graph, memcache_uri: String, memcache_default_expiry: u64) -> Result<Self, HandlerError> {
        let cache_config = CacheConfig::new(vec![memcache_uri])
            .with_default_expiry(Duration::from_secs(memcache_default_expiry));
        let cache = KgCache::new(cache_config)?;

        Ok(Self {
            ipfs: IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/"),
            neo4j,
            cache: Arc::new(cache),
            spaces_blacklist: match blacklist::load() {
                Ok(Some(blacklist)) => {
                    tracing::info!("Blacklisting spaces: {}", blacklist.spaces.join(", "));
                    blacklist.spaces
                }
                Ok(None) => {
                    tracing::info!("No blacklist found");
                    vec![]
                }
                Err(e) => {
                    tracing::warn!("Error loading blacklist, skipping: {:?}", e);
                    vec![]
                }
            },
        })
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

impl substreams_utils::Sink for EventHandler {
    type Error = HandlerError;

    async fn process_block_scoped_data(&self, data: &BlockScopedData) -> Result<(), Self::Error> {
        let _timer = metrics::BLOCK_PROCESSING_TIME.start_timer();

        let output = data.output.as_ref().unwrap().map_output.as_ref().unwrap();

        let block =
            get_block_metadata(data).map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        let drift = chrono::Utc::now().timestamp() - block.timestamp.timestamp();
        metrics::HEAD_BLOCK_TIME_DRIFT.set(drift as f64);
        metrics::HEAD_BLOCK_NUMBER.set(block.block_number as f64);
        metrics::HEAD_BLOCK_TIMESTAMP.set(block.timestamp.timestamp() as f64);

        let value = GeoOutput::decode(output.value.as_slice())?;

        // Handle new space creation
        if !value.spaces_created.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} space created events",
                block.block_number,
                block.timestamp,
                value.spaces_created.len()
            );
        }
        let created_space_ids = stream::iter(&value.spaces_created)
            .then(|event| async {
                self.handle_space_created(event, &value.edits_published, &block)
                    .await
            })
            .try_collect::<Vec<_>>()
            .await?;

        // Handle personal space creation
        if !value.personal_plugins_created.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} personal space created events",
                block.block_number,
                block.timestamp,
                value.personal_plugins_created.len()
            );
        }
        stream::iter(&value.personal_plugins_created)
            .map(Ok)
            .try_for_each(|event| async { self.handle_personal_space_created(event, &block).await })
            .await?;

        // Handle new governance plugin creation
        if !value.governance_plugins_created.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} governance plugin created events",
                block.block_number,
                block.timestamp,
                value.governance_plugins_created.len()
            );
        }
        stream::iter(&value.governance_plugins_created)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_governance_plugin_created(event, &block).await
            })
            .await?;

        if !value.initial_editors_added.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} initial editors added events",
                block.block_number,
                block.timestamp,
                value.initial_editors_added.len()
            );
        }
        stream::iter(&value.initial_editors_added)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_initial_space_editors_added(event, &block).await
            })
            .await?;

        if !value.members_added.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} members added events",
                block.block_number,
                block.timestamp,
                value.members_added.len()
            );
        }
        stream::iter(&value.members_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_member_added(event, &block).await })
            .await?;

        if !value.members_removed.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} members removed events",
                block.block_number,
                block.timestamp,
                value.members_removed.len()
            );
        }
        stream::iter(&value.members_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_member_removed(event, &block).await })
            .await?;

        if !value.editors_added.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} editors added events",
                block.block_number,
                block.timestamp,
                value.editors_added.len()
            );
        }
        stream::iter(&value.editors_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_editor_added(event, &block).await })
            .await?;

        if !value.editors_removed.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} editors removed events",
                block.block_number,
                block.timestamp,
                value.editors_removed.len()
            );
        }
        stream::iter(&value.editors_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_editor_removed(event, &block).await })
            .await?;

        if !value.subspaces_added.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} subspaces added events",
                block.block_number,
                block.timestamp,
                value.subspaces_added.len()
            );
        }
        stream::iter(&value.subspaces_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_subspace_added(event, &block).await })
            .await?;

        if !value.subspaces_removed.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} subspaces removed events",
                block.block_number,
                block.timestamp,
                value.subspaces_removed.len()
            );
        }
        stream::iter(&value.subspaces_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_subspace_removed(event, &block).await })
            .await?;

        if !value.proposed_added_members.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} add member proposal created events",
                block.block_number,
                block.timestamp,
                value.proposed_added_members.len()
            );
        }
        stream::iter(&value.proposed_added_members)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_add_member_proposal_created(event, &block).await
            })
            .await?;

        if !value.proposed_removed_members.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} remove member proposal created events",
                block.block_number,
                block.timestamp,
                value.proposed_removed_members.len()
            );
        }
        stream::iter(&value.proposed_removed_members)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_remove_member_proposal_created(event, &block)
                    .await
            })
            .await?;

        if !value.proposed_added_editors.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} add editor proposal created events",
                block.block_number,
                block.timestamp,
                value.proposed_added_editors.len()
            );
        }
        stream::iter(&value.proposed_added_editors)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_add_editor_proposal_created(event, &block).await
            })
            .await?;

        if !value.proposed_removed_editors.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} remove editor proposal created events",
                block.block_number,
                block.timestamp,
                value.proposed_removed_editors.len()
            );
        }
        stream::iter(&value.proposed_removed_editors)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_remove_editor_proposal_created(event, &block)
                    .await
            })
            .await?;

        if !value.proposed_added_subspaces.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} add subspace proposal created events",
                block.block_number,
                block.timestamp,
                value.proposed_added_subspaces.len()
            );
        }
        stream::iter(&value.proposed_added_subspaces)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_add_subspace_proposal_created(event, &block)
                    .await
            })
            .await?;

        if !value.proposed_removed_subspaces.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} remove subspace proposal created events",
                block.block_number,
                block.timestamp,
                value.proposed_removed_subspaces.len()
            );
        }
        stream::iter(&value.proposed_removed_subspaces)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_remove_subspace_proposal_created(event, &block)
                    .await
            })
            .await?;

        if !value.edits.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} publish edit proposal created events",
                block.block_number,
                block.timestamp,
                value.edits.len()
            );
        }
        stream::iter(&value.edits)
            .map(Ok)
            .try_for_each(|event| async {
                self.handle_publish_edit_proposal_created(event, &block)
                    .await
            })
            .await?;

        if !value.votes_cast.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} vote cast events",
                block.block_number,
                block.timestamp,
                value.votes_cast.len()
            );
        }
        stream::iter(&value.votes_cast)
            .map(Ok)
            .try_for_each(|event| async { self.handle_vote_cast(event, &block).await })
            .await?;

        if !value.edits_published.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} edits published events",
                block.block_number,
                block.timestamp,
                value.edits_published.len()
            );
        }
        self.handle_edits_published(&value.edits_published, &created_space_ids, &block)
            .await?;

        if !value.executed_proposals.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} executed proposal events",
                block.block_number,
                block.timestamp,
                value.executed_proposals.len()
            );
        }
        stream::iter(&value.executed_proposals)
            .enumerate()
            .map(Ok)
            .try_for_each(|(idx, event)| {
                let block = block.clone();
                async move { self.handle_proposal_executed(event, &block, idx).await }
            })
            .await?;

        // Persist block number and timestamp
        grc20_core::mapping::triple::insert_many(
            &self.neo4j,
            &BlockMetadata::default(),
            indexer_ids::INDEXER_SPACE_ID,
            "0",
        )
        .triple(grc20_core::mapping::triple::Triple::new(
            indexer_ids::CURSOR_ID,
            indexer_ids::BLOCK_NUMBER_ATTRIBUTE,
            block.block_number,
        ))
        .triple(grc20_core::mapping::triple::Triple::new(
            indexer_ids::CURSOR_ID,
            indexer_ids::BLOCK_TIMESTAMP_ATTRIBUTE,
            block.timestamp,
        ))
        .send()
        .await?;

        Ok(())
    }

    async fn load_persisted_cursor(&self) -> Result<Option<String>, Self::Error> {
        let cursor = grc20_core::mapping::triple::find_one(
            &self.neo4j,
            indexer_ids::CURSOR_ATTRIBUTE,
            indexer_ids::CURSOR_ID,
            indexer_ids::INDEXER_SPACE_ID,
            Some("0".to_string()),
        )
        .send()
        .await?;

        Ok(cursor.map(|c| c.value.value))
    }

    async fn persist_cursor(&self, cursor: String) -> Result<(), Self::Error> {
        grc20_core::mapping::triple::Triple::new(
            indexer_ids::CURSOR_ID,
            indexer_ids::CURSOR_ATTRIBUTE,
            cursor,
        )
        .insert(
            &self.neo4j,
            &BlockMetadata::default(),
            indexer_ids::INDEXER_SPACE_ID,
            "0",
        )
        .send()
        .await?;

        Ok(())
    }
}
