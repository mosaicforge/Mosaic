use futures::{stream, StreamExt, TryStreamExt};
use grc20_core::{
    block::BlockMetadata,
    entity,
    error::DatabaseError,
    ids,
    mapping::{entity::models::UpdateEntity, relation::models::UpdateRelation},
    network_ids,
    pb::{self, chain},
    property, relation,
};
// use grc20_sdk::models::{
//     self,
//     edit::{Edits, ProposedEdit},
//     space, Proposal,
// };
use ipfs::deserialize;
use uuid::Uuid;
use web3_utils::checksum_address;

use super::{handler::HandlerError, EventHandler};

pub struct Edit {
    pub name: String,
    pub proposal_id: Uuid,
    pub space_id: Uuid,
    pub ops: Vec<pb::grc20::Op>,
}

/// Generates a unique ID for a space based on its network and DAO contract address.
pub fn new_space_id(network: &str, address: &str) -> Uuid {
    ids::create_id_from_unique_string(format!("{network}:{}", checksum_address(address)))
}

impl EventHandler {
    pub async fn handle_edits_published(
        &self,
        edits_published: Vec<(chain::EditPublished, Vec<Edit>)>,
        _created_space_ids: &[Uuid],
        block: &BlockMetadata,
    ) -> Result<(), HandlerError> {
        let edits = edits_published
            .into_iter()
            .flat_map(|(_, edits)| edits)
            .collect::<Vec<_>>();

        // let space_id = Space::new_id(network_ids::GEO, address)

        // TODO: Create "synthetic" proposals for newly created spaces and
        // personal spaces

        stream::iter(edits)
            .enumerate()
            .map(Ok) // Need to wrap the proposal in a Result to use try_for_each
            .try_for_each(
                |(idx, proposal)| async move { self.process_edit(block, proposal, idx).await },
            )
            .await
            .map_err(|e| HandlerError::Other(format!("{e:?}").into()))?; // TODO: Convert anyhow::Error to HandlerError properly

        Ok(())
    }

    pub async fn fetch_edit(
        &self,
        edit_published: &chain::EditPublished,
    ) -> Result<Vec<Edit>, HandlerError> {
        let space_id = new_space_id(network_ids::GEO, &edit_published.dao_address);

        let bytes = self
            .ipfs
            .get_bytes(&edit_published.content_uri.replace("ipfs://", ""), true)
            .await?;

        let edit = if let Ok(edit) = deserialize::<pb::grc20::Edit>(&bytes) {
            edit
        } else {
            tracing::warn!("Invalid metadata for edit {}", edit_published.content_uri);
            return Ok(vec![]);
        };

        Ok(vec![Edit {
            name: edit.name,
            proposal_id: Uuid::from_bytes(
                edit.id
                    .try_into()
                    .map_err(|e| HandlerError::InvalidUuid(format!("{e:?}")))?,
            ),
            space_id: space_id.clone(),
            ops: edit.ops,
        }])

        // match edit.payload {
        //     Some(pb::grc20::file::Payload::AddEdit(edit)) => Ok(vec![Edit {
        //         name: edit.name,
        //         proposal_id: Uuid::from_bytes(
        //             edit.id
        //                 .try_into()
        //                 .map_err(|e| HandlerError::InvalidUuid(format!("{e:?}")))?,
        //         ),
        //         space_id: space_id.clone(),
        //         ops: edit.ops,
        //     }]),
        //     Some(pb::grc20::file::Payload::ImportSpace(import)) => {
        //         stream::iter(import.edits)
        //             .map(|edit_uri| {
        //                 let space_id = space_id.clone();
        //                 // let space_plugin_address = edit_published.plugin_address.clone();

        //                 async move {
        //                     let hash = edit_uri.replace("ipfs://", "");
        //                     let edit = self.ipfs.get::<pb::grc20::Edit>(&hash, true).await?;

        //                     Ok(Edit {
        //                         name: edit.name,
        //                         proposal_id: Uuid::from_bytes(
        //                             edit.id
        //                                 .try_into()
        //                                 .map_err(|e| HandlerError::InvalidUuid(format!("{e:?}")))?,
        //                         ),
        //                         space_id,
        //                         ops: edit.ops,
        //                     })
        //                 }
        //             })
        //             .buffered(16)
        //             .try_collect::<Vec<_>>()
        //             .await
        //     }
        //     _ => Ok(vec![]),
        // }
    }

    pub async fn process_edit(
        &self,
        block: &BlockMetadata,
        edit: Edit,
        _index: usize,
    ) -> Result<(), DatabaseError> {
        // TODO: Store edit metadata
        // 1. Check if edit exists (i.e.: was created via edit proposal)
        // 2. If exists, update edit metadata
        // 3. If not, create edit metadata

        if self.spaces_blacklist.contains(&edit.space_id) {
            tracing::warn!(
                "Block #{} ({}): Space {} is blacklisted, skipping edit",
                block.block_number,
                block.timestamp,
                edit.space_id
            );
            return Ok(());
        }

        // Group ops by type
        let num_ops = edit.ops.len();
        let op_groups = OpGroups::from_ops(edit.ops);

        tracing::info!(
            "Block #{} ({}): Processing {} ops for proposal {}: {} update entities, {} create relations, {} update relations, {} delete relations, {} create properties, {} unset entity values, {} unset relation fields",
            block.block_number,
            block.timestamp,
            num_ops,
            edit.proposal_id,
            op_groups.update_entities.len(),
            op_groups.create_relations.len(),
            op_groups.update_relations.len(),
            op_groups.delete_relations.len(),
            op_groups.create_properties.len(),
            op_groups.unset_entity_values.len(),
            op_groups.unset_relation_fields.len(),
        );

        // Process entity updates
        if !op_groups.update_entities.is_empty() {
            let update_entities: Result<Vec<UpdateEntity>, _> =
                op_groups
                    .update_entities
                    .into_iter()
                    .map(UpdateEntity::try_from)
                    .map(|result| {
                        if let Ok(mut update_entity) = result {
                            // Check if the entity contains a NAME_ATTRIBUTE value
                            if let Some(name_value) = update_entity.values.iter().find(|value| {
                                value.property == grc20_core::system_ids::NAME_ATTRIBUTE
                            }) {
                                // Generate embedding using the handler's embedding model
                                if let Some(embedding) = self
                                    .embedding_model
                                    .embed(vec![&name_value.value], None)
                                    .ok()
                                    .map(|mut embeds| embeds.pop())
                                    .flatten()
                                {
                                    update_entity.embedding = Some(embedding);
                                }
                            }

                            Ok(update_entity)
                        } else {
                            result
                        }
                    })
                    .collect();

            match update_entities {
                Ok(entities) => {
                    entity::update_many(self.neo4j.clone(), edit.space_id)
                        .updates(entities)
                        .send()
                        .await
                        .map_err(|e| DatabaseError::Neo4jError(e))?;
                }
                Err(e) => {
                    tracing::error!("Failed to convert entities: {:?}", e);
                }
            }
        }

        // Process relation creations
        if !op_groups.create_relations.is_empty() {
            let relations: Result<Vec<relation::models::Relation>, _> = op_groups
                .create_relations
                .into_iter()
                .map(|pb_relation| relation::models::Relation::try_from(pb_relation))
                .collect();

            match relations {
                Ok(relations) => {
                    relation::insert_many(self.neo4j.clone())
                        .relations(relations)
                        .send()
                        .await
                        .map_err(|e| DatabaseError::Neo4jError(e))?;
                }
                Err(e) => {
                    tracing::error!("Failed to convert relations: {:?}", e);
                }
            }
        }

        // Process relation updates
        if !op_groups.update_relations.is_empty() {
            let relation_updates: Result<Vec<UpdateRelation>, _> = op_groups
                .update_relations
                .into_iter()
                .map(UpdateRelation::try_from)
                .collect();

            match relation_updates {
                Ok(updates) => {
                    relation::update_many(self.neo4j.clone(), updates)
                        .send()
                        .await
                        .map_err(|e| DatabaseError::Neo4jError(e))?;
                }
                Err(e) => {
                    tracing::error!("Failed to convert relation updates: {:?}", e);
                }
            }
        }

        // Process relation deletions
        if !op_groups.delete_relations.is_empty() {
            relation::delete_many(self.neo4j.clone(), op_groups.delete_relations)
                .send()
                .await
                .map_err(|e| DatabaseError::Neo4jError(e))?;
        }

        // Process property creations
        if !op_groups.create_properties.is_empty() {
            let properties: Result<Vec<property::models::Property>, _> = op_groups
                .create_properties
                .into_iter()
                .map(property::models::Property::try_from)
                .collect();

            match properties {
                Ok(properties) => {
                    property::insert_many(self.neo4j.clone())
                        .properties(properties)
                        .send()
                        .await
                        .map_err(|e| DatabaseError::Neo4jError(e))?;
                }
                Err(e) => {
                    tracing::error!("Failed to convert properties: {:?}", e);
                }
            }
        }

        // Process entity value unsets
        if !op_groups.unset_entity_values.is_empty() {
            let unset_values: Result<Vec<entity::models::UnsetEntityValues>, _> = op_groups
                .unset_entity_values
                .into_iter()
                .map(entity::models::UnsetEntityValues::try_from)
                .collect();

            match unset_values {
                Ok(unset_values) => {
                    entity::update_many(self.neo4j.clone(), edit.space_id)
                        .updates(unset_values)
                        .send()
                        .await
                        .map_err(|e| DatabaseError::Neo4jError(e))?;
                }
                Err(e) => {
                    tracing::error!("Failed to convert unset entity values: {:?}", e);
                }
            }
        }

        // Process relation field unsets
        if !op_groups.unset_relation_fields.is_empty() {
            let unset_fields: Result<Vec<relation::models::UnsetRelationFields>, _> = op_groups
                .unset_relation_fields
                .into_iter()
                .map(relation::models::UnsetRelationFields::try_from)
                .collect();

            match unset_fields {
                Ok(unset_fields) => {
                    relation::update_many(self.neo4j.clone(), unset_fields)
                        .send()
                        .await
                        .map_err(|e| DatabaseError::Neo4jError(e))?;
                }
                Err(e) => {
                    tracing::error!("Failed to convert unset relation fields: {:?}", e);
                }
            }
        }

        Ok(())
    }
}

// Ops are grouped by type
#[derive(Debug, Default)]
pub struct OpGroups {
    update_entities: Vec<pb::grc20::Entity>,
    create_relations: Vec<pb::grc20::Relation>,
    update_relations: Vec<pb::grc20::RelationUpdate>,
    delete_relations: Vec<Uuid>,
    create_properties: Vec<pb::grc20::Property>,
    unset_entity_values: Vec<pb::grc20::UnsetEntityValues>,
    unset_relation_fields: Vec<pb::grc20::UnsetRelationFields>,
}

impl OpGroups {
    pub fn from_ops(ops: impl IntoIterator<Item = pb::grc20::Op>) -> Self {
        let mut op_groups = Self::default();

        for op in ops {
            if let Some(payload) = op.payload {
                use pb::grc20::op::Payload;
                match payload {
                    Payload::UpdateEntity(entity) => {
                        op_groups.update_entities.push(entity);
                    }
                    Payload::CreateRelation(relation) => {
                        op_groups.create_relations.push(relation);
                    }
                    Payload::UpdateRelation(relation_update) => {
                        op_groups.update_relations.push(relation_update);
                    }
                    Payload::DeleteRelation(relation_id_bytes) => {
                        if let Ok(uuid) = Uuid::from_slice(&relation_id_bytes) {
                            op_groups.delete_relations.push(uuid);
                        } else {
                            tracing::warn!(
                                "Invalid UUID for delete relation: {:?}",
                                relation_id_bytes
                            );
                        }
                    }
                    Payload::CreateProperty(property) => {
                        op_groups.create_properties.push(property);
                    }
                    Payload::UnsetEntityValues(unset_values) => {
                        op_groups.unset_entity_values.push(unset_values);
                    }
                    Payload::UnsetRelationFields(unset_fields) => {
                        op_groups.unset_relation_fields.push(unset_fields);
                    }
                }
            } else {
                tracing::warn!("Op has no payload: {:?}", op);
            }
        }

        op_groups
    }
}
