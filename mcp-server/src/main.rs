use clap::{Args, Parser};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use futures::TryStreamExt;
use grc20_core::{
    entity::{self, Entity, EntityRelationFilter},
    mapping::{query_utils::TypesFilter, Query, QueryStream},
    neo4rs, system_ids,
};
use grc20_sdk::models::BaseEntity;
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StructRequest {
    pub a: i32,
    pub b: i32,
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
            .limit(8)
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
                        "types": result.entity.types,
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
                        .to_id(system_ids::RELATION),
                ),
            )
            .limit(8)
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
                        "types": result.entity.types,
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
            .limit(8)
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
                        "types": result.entity.types,
                    }))
                    .expect("Failed to create JSON content")
                })
                .collect(),
        ))
    }

    // #[tool(description = "Search Properties")]
    // async fn get_entities(
    //     &self,
    //     #[tool(param)]
    //     #[schemars(description = "The query string to search for properties")]
    //     query: String,
    // )

    #[tool(description = "Get entity by ID")]
    async fn get_entity(
        &self,
        #[tool(param)]
        #[schemars(description = "Return an entity by its ID along with its attributes (name, description, etc.) and types")]
        id: String,
    ) -> Result<CallToolResult, McpError> {
        let entity = entity::find_one::<Entity<BaseEntity>>(&self.neo4j, &id)
            .send()
            .await
            .map_err(|e| {
                McpError::internal_error(
                    "get_entity",
                    Some(json!({ "error": e.to_string() })),
                )
            })?
            .ok_or_else(|| {
                McpError::internal_error("entity_not_found", Some(json!({ "id": id })))
            })?;

        tracing::info!("Found entity with ID '{}'", id);

        Ok(CallToolResult::success(vec![Content::json(
            json!({
                "id": entity.id(),
                "name": entity.attributes.name,
                "description": entity.attributes.description,
                "types": entity.types,
            }),
        )
        .expect("Failed to create JSON content")]))
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
