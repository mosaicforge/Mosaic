use futures::{stream, StreamExt, TryStreamExt};
use serde::Deserialize;

use crate::{
    bootstrap,
    neo4j_utils::Neo4jExt,
    ops::{conversions, ops::Op},
};

use kg_core::{
    models::{EditProposal, Space},
    system_ids,
};

const ROOT_SPACE_ID: &str = "ab7d4b9e02f840dab9746d352acb0ac6";

#[derive(Clone)]
pub struct Client {
    pub neo4j: neo4rs::Graph,
}

impl Client {
    pub async fn new(uri: &str, user: &str, pass: &str) -> anyhow::Result<Self> {
        let neo4j = neo4rs::Graph::new(uri, user, pass).await?;
        Ok(Self { neo4j })
    }

    /// Bootstrap the database with the initial data
    pub async fn bootstrap(&self, rollup: bool) -> anyhow::Result<()> {
        let bootstrap_ops = if rollup {
            conversions::batch_ops(bootstrap::bootstrap())
        } else {
            bootstrap::bootstrap().into_iter().map(Op::from).collect()
        };

        stream::iter(bootstrap_ops)
            .map(|op| Ok(op)) // Convert to Result to be able to use try_for_each
            .try_for_each(|op| async move { op.apply_op(self, ROOT_SPACE_ID).await })
            .await?;

        Ok(())
    }

    /// Reset the database by deleting all nodes and relations and re-bootstrapping it
    pub async fn reset_db(&self, rollup: bool) -> anyhow::Result<()> {
        // Delete all nodes and relations
        let mut txn = self.neo4j.start_txn().await?;
        txn.run(neo4rs::query("MATCH (n) DETACH DELETE n")).await?;
        txn.commit().await?;

        // Re-bootstrap the database
        self.bootstrap(rollup).await?;

        Ok(())
    }

    pub async fn create_space(&self, space: Space) -> anyhow::Result<()> {
        self.neo4j.insert_one(space).await
    }

    pub async fn get_space_by_address(&self, space_address: &str) -> anyhow::Result<Option<Space>> {
        let query = neo4rs::query("MATCH (s:`Space`) WHERE s.contract_address = $address RETURN s")
            .param("address", space_address);

        self.neo4j.find_one(query).await
    }

    pub async fn find_node_by_id<T: for<'a> Deserialize<'a> + Send>(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<T>> {
        let query = neo4rs::query("MATCH (n { id: $id }) RETURN n").param("id", id);
        self.neo4j.find_one(query).await
    }

    pub async fn find_relation_by_id<T: for<'a> Deserialize<'a> + Send>(
        &self,
        id: &str,
    ) -> anyhow::Result<Option<T>> {
        let query = neo4rs::query("MATCH () -[r]-> () WHERE r.id = $id RETURN r").param("id", id);
        self.neo4j.find_one(query).await
    }

    pub async fn get_name(&self, entity_id: &str) -> anyhow::Result<Option<String>> {
        match self
            .neo4j
            .find_one::<Entity>(Entity::find_by_id_query(entity_id))
            .await?
        {
            Some(Entity {
                name: Some(name), ..
            }) => Ok(Some(name)),
            None | Some(Entity { name: None, .. }) => Ok(None),
        }
    }

    pub async fn find_types<T: for<'a> Deserialize<'a> + Send>(&self) -> anyhow::Result<Vec<T>> {
        let query = neo4rs::query(&format!("MATCH (t:`{}`) RETURN t", system_ids::SCHEMA_TYPE));
        self.neo4j.find_all(query).await
    }

    pub async fn process_edit(&self, edit: EditProposal) -> anyhow::Result<()> {
        let space_id = edit.space.as_str();
        let rolled_up_ops = conversions::batch_ops(edit.ops);

        stream::iter(rolled_up_ops)
            .map(|op| Ok(op)) // Convert to Result to be able to use try_for_each
            .try_for_each(|op| async move { op.apply_op(self, space_id).await })
            .await?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub cover: Option<String>,
    pub content: Option<String>,
}

impl Entity {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: Some(name.to_string()),
            description: None,
            cover: None,
            content: None,
        }
    }

    pub fn find_by_id_query(id: &str) -> neo4rs::Query {
        neo4rs::query("MATCH (n { id: $id }) RETURN n").param("id", id)
    }
}
