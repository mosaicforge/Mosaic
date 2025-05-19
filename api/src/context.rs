use cache::KgCache;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use grc20_core::neo4rs;
use std::sync::Arc;

const EMBEDDING_MODEL: EmbeddingModel = EmbeddingModel::AllMiniLML6V2;

#[derive(Clone)]
pub struct KnowledgeGraph {
    pub neo4j: Arc<neo4rs::Graph>,
    pub cache: Option<Arc<KgCache>>,
    pub embedding_model: Arc<TextEmbedding>,
}

impl juniper::Context for KnowledgeGraph {}

impl KnowledgeGraph {
    pub fn new(neo4j: Arc<neo4rs::Graph>, cache: Option<Arc<KgCache>>) -> Self {
        Self {
            neo4j,
            cache,
            embedding_model: Arc::new(
                TextEmbedding::try_new(
                    InitOptions::new(EMBEDDING_MODEL).with_show_download_progress(true),
                )
                .expect("Failed to initialize embedding model"),
            ),
        }
    }
}
