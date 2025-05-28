use chrono::DateTime;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::{stream, StreamExt, TryStreamExt};
use grc20_core::{
    block::BlockMetadata, error::DatabaseError, ids::create_geo_id, indexer_ids, mapping::Query,
    neo4rs, pb::geo::GeoOutput,
};
use ipfs::IpfsClient;
use prost::Message;
use substreams_utils::pb::sf::substreams::rpc::v2::BlockScopedData;

use crate::{
    blacklist, metrics,
    preprocess::{self, EventData},
};
use cache::KgCache;
use std::sync::Arc;

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

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
    #[allow(dead_code)]
    pub(crate) cache: Option<Arc<KgCache>>,
    pub(crate) spaces_blacklist: Vec<String>,
    pub(crate) embedding_model: fastembed::TextEmbedding,
    pub(crate) embedding_model_dim: usize,

    // Handler config
    pub(crate) versioning: bool,
    pub(crate) governance: bool,
}

impl EventHandler {
    pub fn neo4j(&self) -> &neo4rs::Graph {
        &self.neo4j
    }

    pub fn embedding_dim(&self) -> usize {
        self.embedding_model_dim
    }

    pub fn new(neo4j: neo4rs::Graph, cache: Option<Arc<KgCache>>) -> Result<Self, HandlerError> {
        Self::new_with_ipfs(
            neo4j,
            IpfsClient::from_url("https://gateway.lighthouse.storage/ipfs/"),
            cache,
        )
    }

    pub fn new_with_ipfs(
        neo4j: neo4rs::Graph,
        ipfs: IpfsClient,
        cache: Option<Arc<KgCache>>,
    ) -> Result<Self, HandlerError> {
        Ok(Self {
            ipfs,
            neo4j,
            cache,
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
            embedding_model: TextEmbedding::try_new(
                InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
            )
            .map_err(|e| {
                tracing::error!("Error initializing embedding model: {:?}", e);
                HandlerError::Other(format!("Error initializing embedding model: {:?}", e).into())
            })?,
            embedding_model_dim: TextEmbedding::get_model_info(&EMBEDDING_MODEL)
                .map_err(|e| {
                    tracing::error!("Error getting embedding model info: {:?}", e);
                    HandlerError::Other(
                        format!("Error getting embedding model info: {:?}", e).into(),
                    )
                })?
                .dim,
            versioning: false,
            governance: false,
        })
    }

    pub fn versioning(mut self, versioning: bool) -> Self {
        self.versioning = versioning;
        self
    }

    pub fn governance(mut self, governance: bool) -> Self {
        self.governance = governance;
        self
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

impl substreams_utils::Sink<preprocess::EventData> for EventHandler {
    type Error = HandlerError;

    async fn preprocess_block_scoped_data(
        &self,
        raw_block: &BlockScopedData,
    ) -> Result<preprocess::EventData, Self::Error> {
        let output = raw_block
            .output
            .as_ref()
            .unwrap()
            .map_output
            .as_ref()
            .unwrap();

        let block = get_block_metadata(raw_block)
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;

        let data = GeoOutput::decode(output.value.as_slice())?;

        let prefetched_edits = stream::iter(data.edits_published)
            .then(|edit_event| async {
                let edit = self
                    .fetch_edit(&edit_event)
                    .await
                    .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?;
                Result::<_, Self::Error>::Ok((edit_event, edit))
            })
            .try_collect::<Vec<_>>()
            .await?;

        Ok(EventData {
            block,
            spaces_created: data.spaces_created,
            governance_plugins_created: data.governance_plugins_created,
            initial_editors_added: data.initial_editors_added,
            votes_cast: data.votes_cast,
            edits_published: prefetched_edits,
            successor_spaces_created: data.successor_spaces_created,
            subspaces_added: data.subspaces_added,
            subspaces_removed: data.subspaces_removed,
            executed_proposals: data.executed_proposals,
            members_added: data.members_added,
            editors_added: data.editors_added,
            personal_plugins_created: data.personal_plugins_created,
            members_removed: data.members_removed,
            editors_removed: data.editors_removed,
            edits: data.edits,
            proposed_added_members: data.proposed_added_members,
            proposed_removed_members: data.proposed_removed_members,
            proposed_added_editors: data.proposed_added_editors,
            proposed_removed_editors: data.proposed_removed_editors,
            proposed_added_subspaces: data.proposed_added_subspaces,
            proposed_removed_subspaces: data.proposed_removed_subspaces,
        })
    }

    async fn process_block_scoped_data(
        &self,
        _raw_block: &BlockScopedData,
        data: preprocess::EventData,
    ) -> Result<(), Self::Error> {
        let _timer = metrics::BLOCK_PROCESSING_TIME.start_timer();

        let drift = chrono::Utc::now().timestamp() - data.block.timestamp.timestamp();
        metrics::HEAD_BLOCK_TIME_DRIFT.set(drift as f64);
        metrics::HEAD_BLOCK_NUMBER.set(data.block.block_number as f64);
        metrics::HEAD_BLOCK_TIMESTAMP.set(data.block.timestamp.timestamp() as f64);

        // Handle new space creation
        if !data.spaces_created.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} space created events",
                data.block.block_number,
                data.block.timestamp,
                data.spaces_created.len()
            );
        }
        let created_space_ids = stream::iter(&data.spaces_created)
            .then(|event| async {
                self.handle_space_created(event, &data.edits_published, &data.block)
                    .await
            })
            .try_collect::<Vec<_>>()
            .await?;

        if self.governance {
            // Handle personal space creation
            if !data.personal_plugins_created.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} personal space created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.personal_plugins_created.len()
                );
            }
            stream::iter(&data.personal_plugins_created)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_personal_space_created(event, &data.block).await
                })
                .await?;
    
            // Handle new governance plugin creation
            if !data.governance_plugins_created.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} governance plugin created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.governance_plugins_created.len()
                );
            }
            stream::iter(&data.governance_plugins_created)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_governance_plugin_created(event, &data.block)
                        .await
                })
                .await?;
    
            if !data.initial_editors_added.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} initial editors added events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.initial_editors_added.len()
                );
            }
            stream::iter(&data.initial_editors_added)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_initial_space_editors_added(event, &data.block)
                        .await
                })
                .await?;
    
            if !data.members_added.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} members added events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.members_added.len()
                );
            }
            stream::iter(&data.members_added)
                .map(Ok)
                .try_for_each(|event| async { self.handle_member_added(event, &data.block).await })
                .await?;
    
            if !data.members_removed.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} members removed events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.members_removed.len()
                );
            }
            stream::iter(&data.members_removed)
                .map(Ok)
                .try_for_each(|event| async { self.handle_member_removed(event, &data.block).await })
                .await?;
    
            if !data.editors_added.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} editors added events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.editors_added.len()
                );
            }
            stream::iter(&data.editors_added)
                .map(Ok)
                .try_for_each(|event| async { self.handle_editor_added(event, &data.block).await })
                .await?;
    
            if !data.editors_removed.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} editors removed events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.editors_removed.len()
                );
            }
            stream::iter(&data.editors_removed)
                .map(Ok)
                .try_for_each(|event| async { self.handle_editor_removed(event, &data.block).await })
                .await?;
        }

        if !data.subspaces_added.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} subspaces added events",
                data.block.block_number,
                data.block.timestamp,
                data.subspaces_added.len()
            );
        }
        stream::iter(&data.subspaces_added)
            .map(Ok)
            .try_for_each(|event| async { self.handle_subspace_added(event, &data.block).await })
            .await?;

        if !data.subspaces_removed.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} subspaces removed events",
                data.block.block_number,
                data.block.timestamp,
                data.subspaces_removed.len()
            );
        }
        stream::iter(&data.subspaces_removed)
            .map(Ok)
            .try_for_each(|event| async { self.handle_subspace_removed(event, &data.block).await })
            .await?;

        if self.governance {

            if !data.proposed_added_members.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} add member proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.proposed_added_members.len()
                );
            }
            stream::iter(&data.proposed_added_members)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_add_member_proposal_created(event, &data.block)
                        .await
                })
                .await?;
    
            if !data.proposed_removed_members.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} remove member proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.proposed_removed_members.len()
                );
            }
            stream::iter(&data.proposed_removed_members)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_remove_member_proposal_created(event, &data.block)
                        .await
                })
                .await?;
    
            if !data.proposed_added_editors.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} add editor proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.proposed_added_editors.len()
                );
            }
            stream::iter(&data.proposed_added_editors)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_add_editor_proposal_created(event, &data.block)
                        .await
                })
                .await?;
    
            if !data.proposed_removed_editors.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} remove editor proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.proposed_removed_editors.len()
                );
            }
            stream::iter(&data.proposed_removed_editors)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_remove_editor_proposal_created(event, &data.block)
                        .await
                })
                .await?;
    
            // Handle proposed add subspace
            if !data.proposed_added_subspaces.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} add subspace proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.proposed_added_subspaces.len()
                );
            }
            stream::iter(&data.proposed_added_subspaces)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_add_subspace_proposal_created(event, &data.block)
                        .await
                })
                .await?;
    
            // Handle remove subspace proposal created
            if !data.proposed_removed_subspaces.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} remove subspace proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.proposed_removed_subspaces.len()
                );
            }
            stream::iter(&data.proposed_removed_subspaces)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_remove_subspace_proposal_created(event, &data.block)
                        .await
                })
                .await?;

                // Handle publish edit proposal created
            if !data.edits.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} publish edit proposal created events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.edits.len()
                );
            }
            stream::iter(&data.edits)
                .map(Ok)
                .try_for_each(|event| async {
                    self.handle_publish_edit_proposal_created(event, &data.block)
                        .await
                })
                .await?;
    
            // Handle vote cast
            if !data.votes_cast.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} vote cast events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.votes_cast.len()
                );
            }
            stream::iter(&data.votes_cast)
                .map(Ok)
                .try_for_each(|event| async { self.handle_vote_cast(event, &data.block).await })
                .await?;
        }

        // Handle edits published
        if !data.edits_published.is_empty() {
            tracing::info!(
                "Block #{} ({}): Processing {} edits published events",
                data.block.block_number,
                data.block.timestamp,
                data.edits_published.len()
            );
        }
        self.handle_edits_published(data.edits_published, &created_space_ids, &data.block)
            .await?;

        if self.governance {
            // Handle proposal executed
            if !data.executed_proposals.is_empty() {
                tracing::info!(
                    "Block #{} ({}): Processing {} executed proposal events",
                    data.block.block_number,
                    data.block.timestamp,
                    data.executed_proposals.len()
                );
            }
            stream::iter(&data.executed_proposals)
                .enumerate()
                .map(Ok)
                .try_for_each(|(idx, event)| {
                    let block_ref = &data.block;
                    async move { self.handle_proposal_executed(event, block_ref, idx).await }
                })
                .await?;
        }

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
            data.block.block_number,
        ))
        .triple(grc20_core::mapping::triple::Triple::new(
            indexer_ids::CURSOR_ID,
            indexer_ids::BLOCK_TIMESTAMP_ATTRIBUTE,
            data.block.timestamp,
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
