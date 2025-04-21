use grc20_core::neo4rs;
use std::sync::Arc;
use cache::KgCache;

#[derive(Clone)]
pub struct KnowledgeGraph {
    pub neo4j: Arc<neo4rs::Graph>,
    pub cache: Arc<KgCache>,
}

impl juniper::Context for KnowledgeGraph {}

impl KnowledgeGraph {
    pub fn new(neo4j: Arc<neo4rs::Graph>, cache: Arc<KgCache>) -> Self {
        Self { neo4j, cache }
    }
}
