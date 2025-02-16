use std::collections::HashMap;

use futures::{stream, StreamExt, TryStreamExt};
use ipfs::deserialize;
use sdk::{
    error::DatabaseError,
    mapping::{query_utils::Query, relation_node, triple},
    models::{self, EditProposal, Space},
    pb::{self, geo},
};
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

impl EventHandler {
    pub async fn handle_edits_published(
        &self,
        edits_published: &[geo::EditPublished],
        _created_space_ids: &[String],
        block: &models::BlockMetadata,
    ) -> Result<(), HandlerError> {
        let proposals = stream::iter(edits_published)
            .then(|proposal| async {
                let edits = self.fetch_edit(proposal).await?;
                anyhow::Ok(edits)
            })
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))? // TODO: Convert anyhow::Error to HandlerError properly
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        // let space_id = Space::new_id(network_ids::GEO, address)

        // TODO: Create "synthetic" proposals for newly created spaces and
        // personal spaces

        stream::iter(proposals)
            .map(Ok) // Need to wrap the proposal in a Result to use try_for_each
            .try_for_each(|proposal| async move {
                tracing::info!(
                    "Block #{} ({}): Processing ops for proposal {}",
                    block.block_number,
                    block.timestamp,
                    proposal.proposal_id
                );

                self.process_ops(block, &proposal.space, proposal.ops).await
            })
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        Ok(())
    }

    async fn fetch_edit(
        &self,
        edit: &geo::EditPublished,
    ) -> Result<Vec<EditProposal>, HandlerError> {
        let space = if let Some(space) =
            Space::find_by_dao_address(&self.neo4j, &edit.dao_address)
                .await
                .map_err(|e| {
                    HandlerError::Other(
                        format!(
                            "Error querying space with plugin address {} {e:?}",
                            checksum_address(&edit.plugin_address)
                        )
                        .into(),
                    )
                })? {
            space
        } else {
            tracing::warn!(
                "Matching space in edit not found for plugin address {}",
                edit.plugin_address
            );
            return Ok(vec![]);
        };

        let bytes = self
            .ipfs
            .get_bytes(&edit.content_uri.replace("ipfs://", ""), true)
            .await?;

        let metadata = if let Ok(metadata) = deserialize::<pb::ipfs::IpfsMetadata>(&bytes) {
            metadata
        } else {
            tracing::warn!("Invalid metadata for edit {}", edit.content_uri);
            return Ok(vec![]);
        };

        match metadata.r#type() {
            pb::ipfs::ActionType::AddEdit => {
                let edit = deserialize::<pb::ipfs::Edit>(&bytes)?;
                Ok(vec![EditProposal {
                    name: edit.name,
                    proposal_id: edit.id,
                    space: space.id.to_string(),
                    space_address: space
                        .attributes
                        .space_plugin_address
                        .clone()
                        .expect("Space plugin address not found"),
                    creator: edit.authors[0].clone(),
                    ops: edit.ops,
                }])
            }
            pb::ipfs::ActionType::ImportSpace => {
                let import = deserialize::<pb::ipfs::Import>(&bytes)?;
                let edits = stream::iter(import.edits)
                    .map(|edit| async move {
                        let hash = edit.replace("ipfs://", "");
                        self.ipfs.get::<pb::ipfs::ImportEdit>(&hash, true).await
                    })
                    .buffer_unordered(10)
                    .try_collect::<Vec<_>>()
                    .await?;

                Ok(edits
                    .into_iter()
                    .map(|edit| EditProposal {
                        name: edit.name,
                        proposal_id: edit.id,
                        space: space.id.to_string(),
                        space_address: space
                            .attributes
                            .space_plugin_address
                            .clone()
                            .expect("Space plugin address not found"),
                        creator: edit.authors[0].clone(),
                        ops: edit.ops,
                    })
                    .collect())
            }
            _ => Ok(vec![]),
        }
    }

    pub async fn process_ops(
        &self,
        block: &models::BlockMetadata,
        space_id: &str,
        ops: impl IntoIterator<Item = pb::ipfs::Op>,
    ) -> Result<(), DatabaseError> {
        // Group ops by entity and type
        let entity_ops = EntityOps::from_ops(ops);

        for (_, op_groups) in entity_ops.0 {
            // Handle SET_TRIPLE ops
            triple::insert_many(&self.neo4j, block, space_id, 0)
                .triples(
                    op_groups
                        .set_triples
                        .into_iter()
                        .map(|triple| triple.try_into().expect("Failed to convert triple")),
                )
                .send()
                .await?;

            // Handle DELETE_TRIPLE ops
            triple::delete_many(&self.neo4j, block, space_id, 0)
                .triples(
                    op_groups
                        .delete_triples
                        .into_iter()
                        .map(|triple| (triple.entity, triple.attribute)),
                )
                .send()
                .await?;

            // Handle CREATE_RELATION ops
            relation_node::insert_many(&self.neo4j, block, space_id, 0)
                .relations(
                    op_groups
                        .create_relations
                        .into_iter()
                        .map(|relation| relation.into()),
                )
                .send()
                .await?;

            // Handle DELETE_RELATION ops
            relation_node::delete_many(&self.neo4j, block, space_id, 0)
                .relations(
                    op_groups
                        .delete_relations
                        .into_iter()
                        .map(|relation| relation.id),
                )
                .send()
                .await?;
        }

        Ok(())
    }
}

// Ops are grouped by type
#[derive(Debug, Default)]
pub struct OpGroups {
    set_triples: Vec<pb::ipfs::Triple>,
    delete_triples: Vec<pb::ipfs::Triple>,
    create_relations: Vec<pb::ipfs::Relation>,
    delete_relations: Vec<pb::ipfs::Relation>,
}

pub struct EntityOps(HashMap<String, OpGroups>);

impl EntityOps {
    pub fn from_ops(ops: impl IntoIterator<Item = pb::ipfs::Op>) -> Self {
        let mut entity_ops = HashMap::new();

        for op in ops {
            match (op.r#type(), op) {
                (
                    pb::ipfs::OpType::SetTriple,
                    pb::ipfs::Op {
                        triple: Some(triple),
                        ..
                    },
                ) => {
                    entity_ops
                        .entry(triple.entity.clone())
                        .or_insert_with(|| OpGroups::default())
                        .set_triples
                        .push(triple);
                }
                (
                    pb::ipfs::OpType::DeleteTriple,
                    pb::ipfs::Op {
                        triple: Some(triple),
                        ..
                    },
                ) => {
                    entity_ops
                        .entry(triple.entity.clone())
                        .or_insert_with(|| OpGroups::default())
                        .delete_triples
                        .push(triple);
                }

                (
                    pb::ipfs::OpType::CreateRelation,
                    pb::ipfs::Op {
                        relation: Some(relation),
                        ..
                    },
                ) => {
                    entity_ops
                        .entry(relation.id.clone())
                        .or_insert_with(|| OpGroups::default())
                        .create_relations
                        .push(relation);
                }
                (
                    pb::ipfs::OpType::DeleteRelation,
                    pb::ipfs::Op {
                        relation: Some(relation),
                        ..
                    },
                ) => {
                    entity_ops
                        .entry(relation.id.clone())
                        .or_insert_with(|| OpGroups::default())
                        .delete_relations
                        .push(relation);
                }

                (typ, maybe_triple) => {
                    tracing::warn!("Unhandled case: {:?} {:?}", typ, maybe_triple);
                }
            }
        }

        Self(entity_ops)
    }
}
