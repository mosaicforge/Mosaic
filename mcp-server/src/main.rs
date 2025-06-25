use clap::{Args, Parser};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::{TryStreamExt, future::join_all};
use grc20_core::{
    entity::{
        self, Entity, EntityFilter, EntityNode, EntityRelationFilter, TypesFilter,
        utils::TraverseRelationFilter,
    },
    mapping::{
        Query, QueryStream, RelationEdge, prop_filter, query_utils::RelationDirection, triple,
    },
    neo4rs,
    relation::{self},
    system_ids,
};
use grc20_sdk::models::BaseEntity;
use mcp_server::input_types::{self, SearchTraversalInputFilter};
use rmcp::{
    Error as McpError, RoleServer, ServerHandler,
    model::*,
    service::RequestContext,
    tool,
    transport::sse_server::{SseServer, SseServerConfig},
};
use serde_json::json;
use std::sync::Arc;
use tracing_subscriber::{
    layer::SubscriberExt,
    util::SubscriberInitExt,
    {self},
};

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
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

    #[tool(description = "Search Types")]
    async fn search_types(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for types")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let embedding = self
            .embedding_model
            .embed(vec![&query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let results = entity::search::<Entity<BaseEntity>>(&self.neo4j, embedding)
            .filter(
                entity::EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::SCHEMA_TYPE)),
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_types_failed",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_types_failed",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        tracing::info!("Found {} results for query '{}'", results.len(), query);

        Ok(CallToolResult::success(
            results
                .into_iter()
                .map(|result| {
                    Content::json(json!({
                        "id": result.entity.id(),
                        "name": result.entity.attributes.name,
                        "description": result.entity.attributes.description,
                    }))
                    .expect("Failed to create JSON content")
                })
                .collect(),
        ))
    }

    #[tool(description = "Search Relation Types")]
    async fn search_relation_types(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for relation types")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let embedding = self
            .embedding_model
            .embed(vec![&query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let results = entity::search::<Entity<BaseEntity>>(&self.neo4j, embedding)
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
            .map_err(|e| {
                McpError::internal_error(
                    "search_relation_types",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_relation_types",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        tracing::info!("Found {} results for query '{}'", results.len(), query);

        Ok(CallToolResult::success(
            results
                .into_iter()
                .map(|result| {
                    Content::json(json!({
                        "id": result.entity.id(),
                        "name": result.entity.attributes.name,
                        "description": result.entity.attributes.description,
                    }))
                    .expect("Failed to create JSON content")
                })
                .collect(),
        ))
    }

    #[tool(description = "Search Space")]
    async fn search_space(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for space")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let embedding = self
            .embedding_model
            .embed(vec![&query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let results = entity::search::<Entity<BaseEntity>>(&self.neo4j, embedding)
            .filter(
                entity::EntityFilter::default().relations(
                    EntityRelationFilter::default()
                        .relation_type(system_ids::TYPES_ATTRIBUTE)
                        .to_id(system_ids::SPACE_TYPE),
                ),
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error("search_space", Some(json!({ "error": e.to_string() })))
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error("search_space", Some(json!({ "error": e.to_string() })))
            })?;

        tracing::info!("Found {} results for query '{}'", results.len(), query);

        Ok(CallToolResult::success(
            results
                .into_iter()
                .map(|result| {
                    Content::json(json!({
                        "id": result.entity.id(),
                        "name": result.entity.attributes.name,
                        "description": result.entity.attributes.description,
                    }))
                    .expect("Failed to create JSON content")
                })
                .collect(),
        ))
    }

    #[tool(description = "Search Properties")]
    async fn search_properties(
        &self,
        #[tool(param)]
        #[schemars(description = "The query string to search for properties")]
        query: String,
    ) -> Result<CallToolResult, McpError> {
        let embedding = self
            .embedding_model
            .embed(vec![&query], None)
            .expect("Failed to get embedding")
            .pop()
            .expect("Embedding is empty")
            .into_iter()
            .map(|v| v as f64)
            .collect::<Vec<_>>();

        let results = entity::search::<Entity<BaseEntity>>(&self.neo4j, embedding)
            .filter(
                entity::EntityFilter::default()
                    .relations(TypesFilter::default().r#type(system_ids::ATTRIBUTE)),
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        tracing::info!("Found {} results for query '{}'", results.len(), query);

        Ok(CallToolResult::success(
            results
                .into_iter()
                .map(|result| {
                    Content::json(json!({
                        "id": result.entity.id(),
                        "name": result.entity.attributes.name,
                        "description": result.entity.attributes.description,
                    }))
                    .expect("Failed to create JSON content")
                })
                .collect(),
        ))
    }

    #[tool(
        description = "Get entity from a query over the name and traversals based on relation types"
    )]
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
            .fold(
                entity::search_from_restictions::<Entity<BaseEntity>>(
                    &self.neo4j,
                    embedding.clone(),
                ),
                |query, filter| {
                    query.filter(
                        EntityFilter::default().traverse_relation(
                            TraverseRelationFilter::default()
                                .relation_type_id(filter.relation_type_id)
                                .direction(match filter.direction {
                                    input_types::RelationDirection::From => RelationDirection::From,
                                    input_types::RelationDirection::To => RelationDirection::To,
                                }),
                        ),
                    )
                },
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        let entities_vec: Vec<_> = results_search
            .into_iter()
            .map(|result| {
                json!({
                    "id": result.entity.id(),
                    "name": result.entity.attributes.name,
                    "description": result.entity.attributes.description,
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

    #[tool(
        description = "Get entity from a query over the name and traversals based on relation types names"
    )]
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

        let traversal_filters: Vec<Result<TraverseRelationFilter, McpError>> =
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

                        let rel_results = entity::search::<EntityNode>(&self.neo4j, rel_embedding)
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
                            .map_err(|e| {
                                McpError::internal_error(
                                    "search_relation_types",
                                    Some(json!({ "error": e.to_string() })),
                                )
                            })?
                            .try_collect::<Vec<_>>()
                            .await
                            .map_err(|e| {
                                McpError::internal_error(
                                    "search_relation_types",
                                    Some(json!({ "error": e.to_string() })),
                                )
                            })?;
                        let relation_ids: Vec<String> = rel_results
                            .into_iter()
                            .map(|sem_search| sem_search.entity.id)
                            .collect();
                        Ok(TraverseRelationFilter::default()
                            .direction(match filter.direction {
                                input_types::RelationDirection::From => RelationDirection::From,
                                input_types::RelationDirection::To => RelationDirection::To,
                            })
                            .relation_type_id(prop_filter::value_in(relation_ids)))
                    }))
                    .await
                    .to_vec()
                }
                None => Vec::new(),
            };

        let results_search = traversal_filters
            .into_iter()
            .fold(
                entity::search_from_restictions::<Entity<BaseEntity>>(
                    &self.neo4j,
                    embedding.clone(),
                ),
                |query, result_ids: Result<_, McpError>| match result_ids {
                    Ok(ids) => query.filter(EntityFilter::default().traverse_relation(ids)),
                    Err(_) => query,
                },
            )
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "search_properties",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        let entities_vec: Vec<_> = results_search
            .into_iter()
            .map(|result| {
                json!({
                    "id": result.entity.id(),
                    "name": result.entity.attributes.name,
                    "description": result.entity.attributes.description,
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

    #[tool(description = "Get entity by ID with it's attributes and relations")]
    async fn get_entity_info(
        &self,
        #[tool(param)]
        #[schemars(
            description = "Return an entity by its ID along with its attributes (name, description, etc.), relations and types"
        )]
        id: String,
    ) -> Result<CallToolResult, McpError> {
        let entity_attributes = triple::find_many(&self.neo4j)
            .entity_id(prop_filter::value(&id))
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error("get_entity_info", Some(json!({ "error": e.to_string() })))
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error("get_entity_info", Some(json!({ "error": e.to_string() })))
            })?;

        let out_relations = relation::find_many::<RelationEdge<EntityNode>>(&self.neo4j)
            .filter(
                relation::RelationFilter::default()
                    .from_(EntityFilter::default().id(prop_filter::value(id.clone()))),
            )
            .limit(8)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id_not_found",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        let in_relations = relation::find_many::<RelationEdge<EntityNode>>(&self.neo4j)
            .filter(
                relation::RelationFilter::default()
                    .to_(EntityFilter::default().id(prop_filter::value(id.clone()))),
            )
            .limit(8)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_id_not_found",
                    Some(json!({ "error": e.to_string() })),
                )
            })?;

        tracing::info!("Found entity with ID '{}'", id);

        let clean_up_relations = |relations: Vec<RelationEdge<EntityNode>>, is_inbound: bool| async move {
            join_all(relations
                .into_iter()
                .map(|result| async move {
                    json!({
                        "relation_id": result.id,
                        "relation_type": self.get_name_of_id(result.relation_type).await.unwrap_or("No relation type".to_string()),
                        "id": if is_inbound {result.from.id.clone()} else {result.to.id.clone()},
                        "name": self.get_name_of_id(if is_inbound {result.from.id.clone()} else {result.to.id.clone()}).await.unwrap_or("No name".to_string()),
                    })
                })).await.to_vec()
        };
        let inbound_relations = clean_up_relations(in_relations, true).await;
        let outbound_relations = clean_up_relations(out_relations, false).await;

        let attributes_vec: Vec<_> = join_all(entity_attributes.into_iter().map(
            |attr| async {
                json!({
                    "attribute_name": self.get_name_of_id(attr.attribute).await.unwrap_or("No attribute name".to_string()),
                    "attribute_value": String::try_from(attr.value).unwrap_or("No attributes".to_string()),
                })
            },
        ))
        .await
        .to_vec();

        Ok(CallToolResult::success(vec![
            Content::json(json!({
                "id": id,
                "all_attributes": attributes_vec,
                "inbound_relations": inbound_relations,
                "outbound_relations": outbound_relations,
            }))
            .expect("Failed to create JSON content"),
        ]))
    }

    #[tool(description = "Search for distant or close Relations between 2 entities")]
    async fn get_relations_between_entities(
        &self,
        #[tool(param)]
        #[schemars(description = "The id of the first Entity to find relations")]
        entity1_id: String,
        #[tool(param)]
        #[schemars(description = "The id of the second Entity to find relations")]
        entity2_id: String,
    ) -> Result<CallToolResult, McpError> {
        let relations = entity::find_path(&self.neo4j, entity1_id.clone(), entity2_id.clone())
            .limit(10)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_relation_by_ids",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .into_iter()
            .collect::<Vec<_>>();

        tracing::info!("Found {} relations", relations.len());

        Ok(CallToolResult::success(
            join_all(relations
                .into_iter()
                .map(|result| async {
                    Content::json(json!({
                    "nodes": join_all(result.nodes_ids.into_iter().map(|node_id| async {self.get_name_of_id(node_id).await.unwrap_or("No attribute name".to_string())})).await.to_vec(),
                    "relations": join_all(result.relations_ids.into_iter().map(|node_id| async {self.get_name_of_id(node_id).await.unwrap_or("No attribute name".to_string())})).await.to_vec(),
                    }))
                    .expect("Failed to create JSON content")
                }))
                .await
                .to_vec(),
        ))
    }

    async fn get_name_of_id(&self, id: String) -> Result<String, McpError> {
        let entity = entity::find_one::<Entity<BaseEntity>>(&self.neo4j, &id)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error("get_entity_name", Some(json!({ "error": e.to_string() })))
            })?
            .ok_or_else(|| {
                McpError::internal_error("entity_name_not_found", Some(json!({ "id": id })))
            })?;
        Ok(entity.attributes.name.unwrap_or("No name".to_string()))
    }
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
