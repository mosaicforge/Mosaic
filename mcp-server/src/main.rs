use clap::{Args, Parser};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::{StreamExt, TryStreamExt, future::join_all};
use grc20_core::{
    entity::{
        self, Entity, EntityRelationFilter,
        utils::{EntityFilter, RelationTraversal, TypesFilter},
    },
    mapping::query_utils::value_filter,
    neo4rs, property,
    relation::{self, RelationDirection, models::Relation},
    system_ids,
};
// use grc20_sdk::models::BaseEntity;
use mcp_server::input_types::{self, SearchTraversalInputFilter};
use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    model::*,
    service::RequestContext,
    tool,
    transport::sse_server::{SseServer, SseServerConfig},
};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};
use uuid::Uuid;

const BIND_ADDRESS: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = AppArgs::parse();

    let neo4j = neo4rs::Graph::new(
        &args.neo4j_args.neo4j_uri,
        &args.neo4j_args.neo4j_user,
        &args.neo4j_args.neo4j_pass,
    )
    .await?;

    let config = SseServerConfig {
        bind: BIND_ADDRESS.parse()?,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
        ct: tokio_util::sync::CancellationToken::new(),
        sse_keep_alive: None,
    };

    let (sse_server, router) = SseServer::new(config);

    // Do something with the router, e.g., add routes or middleware

    let listener = tokio::net::TcpListener::bind(sse_server.config.bind).await?;

    let ct = sse_server.config.ct.child_token();

    let server = axum::serve(listener, router).with_graceful_shutdown(async move {
        ct.cancelled().await;
        tracing::info!("sse server cancelled");
    });

    tokio::spawn(async move {
        if let Err(e) = server.await {
            tracing::error!(error = %e, "sse server shutdown with error");
        }
    });

    let ct = sse_server.with_service(move || KnowledgeGraph::new(neo4j.clone()));

    tokio::signal::ctrl_c().await?;
    ct.cancel();
    Ok(())
}

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

#[derive(Clone)]
pub struct KnowledgeGraph {
    neo4j: neo4rs::Graph,
    pub embedding_model: Arc<TextEmbedding>,
}

#[tool(tool_box)]
impl KnowledgeGraph {
    #[allow(dead_code)]
    pub fn new(neo4j: neo4rs::Graph) -> Self {
        Self {
            neo4j,
            embedding_model: Arc::new(
                TextEmbedding::try_new(
                    InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
                )
                .expect("Failed to initialize embedding model"),
            ),
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        RawResource::new(uri, name.to_string()).no_annotation()
    }

    fn embed_query(&self, query: &str) -> Result<Vec<f64>, McpError> {
        self.embedding_model
            .embed(vec![query], None)
            .map_err(|e| {
                McpError::internal_error(
                    "embedding_failed",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .pop()
            .ok_or_else(|| {
                McpError::internal_error(
                    "embedding_failed",
                    Some(json!({ "error": "Embedding is empty" })),
                )
            })
            .map(|embedding| embedding.into_iter().map(|v| v as f64).collect::<Vec<_>>())
    }

    async fn search(
        &self,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<entity::SemanticSearchResult>, McpError> {
        let embedding = self
            .embedding_model
            .embed(vec![&query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let limit = limit.unwrap_or(10);

        let semantic_search_triples = entity::search(&self.neo4j, embedding)
            .limit(limit)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        Ok(semantic_search_triples)
    }

    // async fn get_ids_from_search(
    //     &self,
    //     search_triples: Vec<SemanticSearchResult>,
    //     create_relation_filter: impl Fn(SemanticSearchResult) -> RelationFilter,
    // ) -> Result<Vec<String>, McpError> {
    //     let mut seen_ids: HashSet<String> = HashSet::new();
    //     let mut result_ids: Vec<String> = Vec::new();

    //     for semantic_search_triple in search_triples {
    //         let filtered_for_types = relation::find_many(&self.neo4j)
    //             .filter(create_relation_filter(semantic_search_triple))
    //             .send()
    //             .await;

    //         //We only need to get the first relation since they would share the same entity id
    //         if let Ok(stream) = filtered_for_types {
    //             pin_mut!(stream);
    //             if let Some(edge) = stream.try_next().await.ok().flatten() {
    //                 let id = edge.from.id;
    //                 if seen_ids.insert(id.clone()) {
    //                     result_ids.push(id);
    //                 }
    //             }
    //         }
    //     }
    //     Ok(result_ids)
    // }

    async fn format_entity(&self, entity: Entity) -> Result<serde_json::Value, McpError> {
        let properties = entity
            .get_properties(&self.neo4j)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .map_ok(|prop| (prop.id, prop))
            .try_collect::<HashMap<_, _>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        Ok(json!({
            "entity_id": entity.id,
            "properties": entity.flattened_properties().into_iter()
                .map(|(prop_id, values)| json!({
                    "name": properties.get(&prop_id).map(|e| e.names().join(",")).unwrap_or("UNKNOWN".to_string()),
                    "values": values.into_iter().map(|v| v.value).collect::<Vec<_>>()
                }))
                .collect::<Vec<_>>()
        }))
    }

    #[tool(description = include_str!("../resources/search_type_description.md"))]
    async fn search_types(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for types")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let vector = self.embed_query(&query)?;

        // let semantic_search_triples = self.search(query, Some(10)).await.unwrap_or_default();

        // let create_relation_filter = |search_result: SemanticSearchResult| {
        //     RelationFilter::default()
        //         .from_(EntityFilter::default().id(prop_filter::value(search_result.triple.entity)))
        //         .relation_type(
        //             EntityFilter::default().id(prop_filter::value(system_ids::TYPES_ATTRIBUTE)),
        //         )
        //         .to_(EntityFilter::default().id(prop_filter::value(system_ids::SCHEMA_TYPE)))
        // };

        // let result_types = self
        //     .get_ids_from_search(semantic_search_triples, &create_relation_filter)
        //     .await
        //     .unwrap_or_default();
        //
        let types = entity::exact_search(&self.neo4j, vector)
            .filter(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE))
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        // let entities: Vec<Result<Vec<Triple>, McpError>> =
        //     join_all(result_types.into_iter().map(|id| async {
        //         triple::find_many(&self.neo4j)
        //             .entity_id(prop_filter::value(id))
        //             .send()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })?
        //             .try_collect::<Vec<_>>()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })
        //     }))
        //     .await
        //     .to_vec();

        Ok(CallToolResult::success(
            join_all(types.into_iter().map(|result| async {
                Content::json(self.format_entity(result.into_entity()).await)
                    .expect("Failed to create JSON content")
            }))
            .await
            .to_vec(),
        ))
    }

    #[tool(description = include_str!("../resources/search_relation_type_description.md"))]
    async fn search_relation_types(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for relation types")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let vector = self.embed_query(&query)?;

        // let semantic_search_triples = self.search(query, Some(10)).await.unwrap_or_default();

        // let create_relation_filter = |search_result: SemanticSearchResult| {
        //     RelationFilter::default()
        //         .from_(EntityFilter::default().id(prop_filter::value(search_result.triple.entity)))
        //         .relation_type(
        //             EntityFilter::default()
        //                 .id(prop_filter::value(system_ids::VALUE_TYPE_ATTRIBUTE)),
        //         )
        //         .to_(
        //             EntityFilter::default()
        //                 .id(prop_filter::value(system_ids::RELATION_SCHEMA_TYPE)),
        //         )
        // };

        // let result_types = self
        //     .get_ids_from_search(semantic_search_triples, &create_relation_filter)
        //     .await
        //     .unwrap_or_default();
        let relation_types = entity::exact_search(&self.neo4j, vector)
            .filter(TypesFilter::default().r#type(system_ids::PROPERTY_TYPE))
            .data_type(property::DataType::Relation)
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        // let entities: Vec<Result<Vec<Triple>, McpError>> =
        //     join_all(result_types.into_iter().map(|id| async {
        //         triple::find_many(&self.neo4j)
        //             .entity_id(prop_filter::value(id))
        //             .send()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })?
        //             .try_collect::<Vec<_>>()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })
        //     }))
        //     .await
        //     .to_vec();

        Ok(CallToolResult::success(
            join_all(relation_types.into_iter().map(|result| async {
                Content::json(self.format_entity(result.into_entity()).await)
                    .expect("Failed to create JSON content")
            }))
            .await
            .to_vec(),
        ))
    }

    #[tool(description = include_str!("../resources/search_space_description.md"))]
    async fn search_space(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for space")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let vector = self.embed_query(&query)?;

        // let semantic_search_triples = self.search(query, Some(10)).await.unwrap_or_default();

        // let create_relation_filter = |search_result: SemanticSearchResult| {
        //     RelationFilter::default()
        //         .from_(EntityFilter::default().id(prop_filter::value(search_result.triple.entity)))
        //         .relation_type(
        //             EntityFilter::default().id(prop_filter::value(system_ids::TYPES_ATTRIBUTE)),
        //         )
        //         .to_(EntityFilter::default().id(prop_filter::value(system_ids::SPACE_TYPE)))
        // };

        // let result_types = self
        //     .get_ids_from_search(semantic_search_triples, &create_relation_filter)
        //     .await
        //     .unwrap_or_default();

        // let entities: Vec<Result<Vec<Triple>, McpError>> =
        //     join_all(result_types.into_iter().map(|id| async {
        //         triple::find_many(&self.neo4j)
        //             .entity_id(prop_filter::value(id))
        //             .send()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })?
        //             .try_collect::<Vec<_>>()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })
        //     }))
        //     .await
        //     .to_vec();

        let spaces = entity::exact_search(&self.neo4j, vector)
            .filter(TypesFilter::default().r#type(system_ids::SPACE_TYPE))
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        Ok(CallToolResult::success(
            join_all(spaces.into_iter().map(|result| async {
                Content::json(self.format_entity(result.into_entity()).await)
                    .expect("Failed to create JSON content")
            }))
            .await
            .to_vec(),
        ))
    }

    #[tool(description = include_str!("../resources/search_properties_description.md"))]
    async fn search_properties(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for properties")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let vector = self.embed_query(&query)?;

        // let semantic_search_triples = self.search(query, Some(10)).await.unwrap_or_default();

        // let create_relation_filter = |search_result: SemanticSearchResult| {
        //     RelationFilter::default()
        //         .from_(EntityFilter::default().id(prop_filter::value(search_result.triple.entity)))
        //         .relation_type(
        //             EntityFilter::default().id(prop_filter::value(system_ids::TYPES_ATTRIBUTE)),
        //         )
        //         .to_(EntityFilter::default().id(prop_filter::value(system_ids::PROPERTY_TYPE)))
        // };

        // let result_types = self
        //     .get_ids_from_search(semantic_search_triples, &create_relation_filter)
        //     .await
        //     .unwrap_or_default();

        // let entities: Vec<Result<Vec<Triple>, McpError>> =
        //     join_all(result_types.into_iter().map(|id| async {
        //         triple::find_many(&self.neo4j)
        //             .entity_id(prop_filter::value(id))
        //             .send()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })?
        //             .try_collect::<Vec<_>>()
        //             .await
        //             .map_err(|e| {
        //                 McpError::internal_error(
        //                     "search_types_failed",
        //                     Some(json!({ "error": e.to_string() })),
        //                 )
        //             })
        //     }))
        //     .await
        //     .to_vec();

        let properties = entity::exact_search(&self.neo4j, vector)
            .filter(TypesFilter::default().r#type(system_ids::PROPERTY_TYPE))
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        Ok(CallToolResult::success(
            join_all(properties.into_iter().map(|result| async {
                Content::json(self.format_entity(result.into_entity()).await)
                    .expect("Failed to create JSON content")
            }))
            .await
            .to_vec(),
        ))
    }

    #[tool(description = include_str!("../resources/search_entity_description.md"))]
    async fn search_entity(
        &self,
        #[tool(param)]
        #[schemars(description = "A filter of the relation(s) to traverse from the query")]
        search_traversal_filter: SearchTraversalInputFilter,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!(
            "SearchTraversalFilter query: {}",
            search_traversal_filter.query
        );

        let embedding = self
            .embedding_model
            .embed(vec![&search_traversal_filter.query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let traversal_filters: Vec<_> = search_traversal_filter
            .traversal_filter
            .map(|relation_filter| relation_filter.into_iter().collect())
            .unwrap_or_default();

        let results_search = traversal_filters
            .into_iter()
            .try_fold(
                entity::search(&self.neo4j, embedding.clone()),
                |query, filter| {
                    Ok::<_, McpError>(
                        query.traversal(
                            RelationTraversal::default()
                                .relation_type_id(filter.relation_type_id.parse::<Uuid>().map_err(
                                    |e| mcp_server::error::Error::InvalidUuid(e.to_string()),
                                )?)
                                .direction(match filter.direction {
                                    input_types::RelationDirection::From => RelationDirection::From,
                                    input_types::RelationDirection::To => RelationDirection::To,
                                }),
                        ),
                    )
                },
            )?
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        let entities_vec: Vec<_> = results_search
            .into_iter()
            .map(|result| {
                let entity = result.into_entity();
                json!({
                    "id": entity.id,
                    "name": entity.names(),
                    "description": entity.descriptions(),
                })
            })
            .collect::<Vec<_>>();

        Ok(CallToolResult::success(vec![
            Content::json(json!({
                "entities": entities_vec,
            }))
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = include_str!("../resources/name_search_entity_description.md"))]
    async fn name_search_entity(
        &self,
        #[tool(param)]
        #[schemars(description = "A filter of the relation(s) to traverse from the query")]
        search_traversal_filter: SearchTraversalInputFilter,
    ) -> Result<CallToolResult, McpError> {
        tracing::info!("SearchTraversalFilter query: {:?}", search_traversal_filter);

        let embedding = self
            .embedding_model
            .embed(vec![&search_traversal_filter.query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let traversal_filters: Vec<Result<RelationTraversal, McpError>> =
            match search_traversal_filter.traversal_filter {
                Some(traversal_filter) => {
                    join_all(traversal_filter.into_iter().map(|filter| async move {
                        let rel_embedding = self
                            .embedding_model
                            .embed(vec![&filter.relation_type_id], None)
                            .expect("Failed to get embedding")
                            .pop()
                            .expect("Embedding is empty")
                            .into_iter()
                            .map(|v| v as f64)
                            .collect::<Vec<_>>();

                        let rel_results = entity::search(&self.neo4j, rel_embedding)
                            .filter(
                                entity::EntityFilter::default().relations(
                                    EntityRelationFilter::default()
                                        .relation_type(system_ids::VALUE_TYPE_ATTRIBUTE)
                                        .to_id(system_ids::RELATION_SCHEMA_TYPE),
                                ),
                            )
                            .limit(10)
                            .send()
                            .await
                            .map_err(mcp_server::error::Error::from)?
                            .try_collect::<Vec<_>>()
                            .await
                            .map_err(mcp_server::error::Error::from)?;

                        let relation_ids = rel_results
                            .into_iter()
                            .map(|sem_search| sem_search.entity_id)
                            .collect::<Vec<_>>();

                        Ok(RelationTraversal::default()
                            .direction(match filter.direction {
                                input_types::RelationDirection::From => RelationDirection::From,
                                input_types::RelationDirection::To => RelationDirection::To,
                            })
                            .relation_type_id(value_filter::value_in(relation_ids)))
                    }))
                    .await
                    .to_vec()
                }
                None => Vec::new(),
            };

        let results_search = traversal_filters
            .into_iter()
            .fold(
                entity::search(&self.neo4j, embedding.clone()),
                |query, result_ids: Result<_, McpError>| match result_ids {
                    Ok(ids) => query.traversal(ids),
                    Err(_) => query,
                },
            )
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        let entities_vec: Vec<_> = results_search
            .into_iter()
            .map(|result| {
                let entity = result.into_entity();
                json!({
                    "id": entity.id,
                    "name": entity.names(),
                    "description": entity.descriptions(),
                })
            })
            .collect::<Vec<_>>();

        Ok(CallToolResult::success(vec![
            Content::json(json!({
                "entities": entities_vec,
            }))
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = include_str!("../resources/get_entity_info_description.md"))]
    async fn get_entity_info(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Return an entity by its ID along with its attributes (name, description, etc.), relations and types"
        )]
        id: String,
    ) -> Result<CallToolResult, McpError> {
        let id = id
            .parse::<Uuid>()
            .map_err(|e| McpError::invalid_params("Invalid UUID for entity_id", None))?;

        let entity = if let Some(entity) = entity::find_one(&self.neo4j, id)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
        {
            entity
        } else {
            return Err(McpError::resource_not_found(
                "Entity not found",
                Some(json!({ "entity_id": id })),
            ));
        };

        // let entity_attributes = triple::find_many(&self.neo4j)
        //     .entity_id(prop_filter::value(&id))
        //     .send()
        //     .await
        //     .map_err(|e| {
        //         McpError::internal_error("get_entity_info", Some(json!({ "error": e.to_string() })))
        //     })?
        //     .try_collect::<Vec<_>>()
        //     .await
        //     .map_err(|e| {
        //         McpError::internal_error("get_entity_info", Some(json!({ "error": e.to_string() })))
        //     })?;

        let out_relations = relation::find_many(&self.neo4j)
            .from(id.clone())
            // .filter(
            // relation::RelationFilter::default()
            //     .from_(EntityFilter::default().id(prop_filter::value(id.clone()))),
            // )
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        let in_relations = relation::find_many(&self.neo4j)
            .to(id.clone())
            // .filter(
            //     relation::RelationFilter::default()
            //         .to_(EntityFilter::default().id(prop_filter::value(id.clone()))),
            // )
            .limit(10)
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .try_collect::<Vec<_>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        // Function to get the related entities of the relations, i.e.: the entities that the relations point to and the type entity of the relation
        let format_relations = |relations: Vec<Relation>, is_inbound: bool| async move {
            // Get the entities of the relations' types
            let relation_type_entities = entity::find_many(&self.neo4j)
                .id(relations
                    .iter()
                    .map(|r| r.r#type.clone())
                    .collect::<Vec<_>>())
                .send()
                .await
                .map_err(mcp_server::error::Error::from)?
                .map_ok(|entity| (entity.id, entity))
                .try_collect::<HashMap<_, _>>()
                .await
                .map_err(mcp_server::error::Error::from)?;

            // Get the entities of the relations' neighbors
            let neighbors = entity::find_many(&self.neo4j)
                .id(relations
                    .iter()
                    .map(|r| {
                        if is_inbound {
                            r.from_entity.clone()
                        } else {
                            r.to_entity.clone()
                        }
                    })
                    .collect::<Vec<_>>())
                .send()
                .await
                .map_err(mcp_server::error::Error::from)?
                .map_ok(|entity| (entity.id, entity))
                .try_collect::<HashMap<_, _>>()
                .await
                .map_err(mcp_server::error::Error::from)?;

            Ok::<_, McpError>(relations.into_iter()
                .filter_map(|result| {
                    let relation_type = relation_type_entities.get(&result.r#type)?;
                    let neighbor = neighbors.get(
                        if is_inbound {
                            &result.from_entity
                        } else {
                            &result.to_entity
                        },
                    )?;
                    Some(json!({
                        "relation_id": result.id,
                        "relation_type": relation_type.names().join(", "),
                        "id": if is_inbound {result.from_entity.clone()} else {result.to_entity.clone()},
                        "name": neighbor.names().join(", "),
                    }))
                })
                .collect::<Vec<_>>())
        };

        let inbound_relations = format_relations(in_relations, true).await?;
        let outbound_relations = format_relations(out_relations, false).await?;
        let entity_details = self.format_entity(entity).await?;

        Ok(CallToolResult::success(vec![
            Content::json(json!({
                "id": entity_details.get("entity_id").cloned().unwrap_or_default(),
                "properties": entity_details.get("properties").cloned().unwrap_or_default(),
                "inbound_relations": inbound_relations,
                "outbound_relations": outbound_relations,
            }))
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = include_str!("../resources/get_relations_between_entities_description.md"))]
    async fn get_relations_between_entities(
        &self,
        #[tool(param)]
        #[schemars(description = "The id of the first Entity to find relations")]
        entity1_id: String,
        #[tool(param)]
        #[schemars(description = "The id of the second Entity to find relations")]
        entity2_id: String,
    ) -> Result<CallToolResult, McpError> {
        let paths = entity::find_path(
            &self.neo4j,
            entity1_id
                .parse::<Uuid>()
                .map_err(|e| McpError::invalid_params("Invalid UUID for entity1_id", None))?,
            entity2_id
                .parse::<Uuid>()
                .map_err(|e| McpError::invalid_params("Invalid UUID for entity2_id", None))?,
        )
        .limit(10)
        .send()
        .await
        .map_err(mcp_server::error::Error::from)?
        .into_iter()
        .collect::<Vec<_>>();

        tracing::info!("Found {} paths", paths.len());

        let entities = entity::find_many(&self.neo4j)
            .id(paths
                .iter()
                .flat_map(|path| path.nodes_ids.clone())
                .collect::<Vec<_>>())
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .map_ok(|entity| (entity.id, entity))
            .try_collect::<HashMap<_, _>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        let relation_types = entity::find_many(&self.neo4j)
            .id(paths
                .iter()
                .flat_map(|path| path.relation_type_ids.clone())
                .collect::<Vec<_>>())
            .send()
            .await
            .map_err(mcp_server::error::Error::from)?
            .map_ok(|entity| (entity.id, entity))
            .try_collect::<HashMap<_, _>>()
            .await
            .map_err(mcp_server::error::Error::from)?;

        Ok(CallToolResult::success(
            paths
                .into_iter()
                .map(|result| {
                    Content::json(json!({
                        "nodes": result.nodes_ids.into_iter().map(|node_id| entities.get(&node_id).map_or_else(
                            || json!({"id": node_id, "name": "Unknown entity"}),
                            |entity| json!({"id": entity.id, "name": entity.names().join(", ")}),
                        )).collect::<Vec<_>>(),
                        "relations": result.relation_type_ids.into_iter().map(|relation_id| relation_types.get(&relation_id).map_or_else(
                            || json!({"id": relation_id, "name": "Unknown relation type"}),
                            |relation| json!({"id": relation.id, "name": relation.names().join(", ")}),
                        )).collect::<Vec<_>>(),
                    }))
                    .expect("Failed to create JSON content")
                })
                .collect::<Vec<_>>(),
        ))
    }

    // async fn get_name_of_id(&self, id: Uuid) -> Result<String, McpError> {
    //     let entity = entity::find_one(&self.neo4j, id)
    //         .send()
    //         .await
    //         .map_err(|e| {
    //             McpError::internal_error("get_entity_name", Some(json!({ "error": e.to_string() })))
    //         })?
    //         .ok_or_else(|| {
    //             McpError::internal_error("entity_name_not_found", Some(json!({ "id": id })))
    //         })?;

    //     Ok(entity.attributes.name.unwrap_or("No name".to_string()))
    // }
}

#[tool(tool_box)]
impl ServerHandler for KnowledgeGraph {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(include_str!("../resources/instructions.md").to_string()),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }

    //TODO: make prompt examples to use on data
    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "example_prompt",
                Some("This is an example prompt that takes one required argument, message"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }

    async fn get_prompt(
        &self,
        GetPromptRequestParam { name, arguments }: GetPromptRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        match name.as_str() {
            "example_prompt" => {
                let message = arguments
                    .and_then(|json| json.get("message")?.as_str().map(|s| s.to_string()))
                    .ok_or_else(|| {
                        McpError::invalid_params("No message provided to example_prompt", None)
                    })?;

                let prompt =
                    format!("This is an example prompt with your message here: '{message}'");
                Ok(GetPromptResult {
                    description: None,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(prompt),
                    }],
                })
            }
            _ => Err(McpError::invalid_params("prompt not found", None)),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "stdout", version, about, arg_required_else_help = true)]
struct AppArgs {
    #[clap(flatten)]
    neo4j_args: Neo4jArgs,
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
