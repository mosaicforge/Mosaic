use futures::{stream, StreamExt, TryStreamExt};
use serde::Deserialize;

use crate::{
    bootstrap,
    neo4j_utils::{serde_value_to_bolt, Neo4jExt},
    ops::{conversions, op::Op},
};
use web3_utils::checksum_address;

use kg_core::{
    ids,
    models::{self, EditProposal, Space},
    system_ids,
};

use super::mapping::{Node, Relation};

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
            .map(Ok) // Convert to Result to be able to use try_for_each
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

    pub async fn add_space(
        &self,
        block: &models::BlockMetadata,
        space: Space,
    ) -> anyhow::Result<()> {
        self.upsert_node(
            system_ids::INDEXER_SPACE_ID,
            block,
            Node::new(space.id.clone(), space).with_type(system_ids::INDEXED_SPACE),
        )
        .await
    }

    pub async fn get_space_by_dao_address(
        &self,
        dao_address: &str,
    ) -> anyhow::Result<Option<Space>> {
        let query = neo4rs::query(&format!(
            "MATCH (n:`{INDEXED_SPACE}` {{dao_contract_address: $dao_contract_address}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        ))
        .param("dao_contract_address", checksum_address(dao_address, None));

        self.neo4j.find_one(query).await
    }

    pub async fn get_space_by_space_plugin_address(
        &self,
        plugin_address: &str,
    ) -> anyhow::Result<Option<Space>> {
        let query = neo4rs::query(&format!(
            "MATCH (n:`{INDEXED_SPACE}` {{space_plugin_address: $space_plugin_address}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        ))
        .param(
            "space_plugin_address",
            checksum_address(plugin_address, None),
        );

        self.neo4j.find_one(query).await
    }

    pub async fn get_space_by_voting_plugin_address(
        &self,
        voting_plugin_address: &str,
    ) -> anyhow::Result<Option<Space>> {
        let query = neo4rs::query(&format!(
            "MATCH (n:`{INDEXED_SPACE}` {{voting_plugin_address: $voting_plugin_address}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        ))
        .param(
            "voting_plugin_address",
            checksum_address(voting_plugin_address, None),
        );

        self.neo4j.find_one(query).await
    }

    pub async fn get_space_by_member_access_plugin(
        &self,
        member_access_plugin: &str,
    ) -> anyhow::Result<Option<Space>> {
        let query = neo4rs::query(&format!(
            "MATCH (n:`{INDEXED_SPACE}` {{member_access_plugin: $member_access_plugin}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        ))
        .param(
            "member_access_plugin",
            checksum_address(member_access_plugin, None),
        );

        self.neo4j.find_one(query).await
    }

    pub async fn get_space_by_personal_plugin_address(
        &self,
        personal_space_admin_plugin: &str,
    ) -> anyhow::Result<Option<Space>> {
        let query = neo4rs::query(&format!(
            "MATCH (n:`{INDEXED_SPACE}` {{personal_space_admin_plugin: $personal_space_admin_plugin}}) RETURN n",
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
        ))
        .param(
            "personal_space_admin_plugin",
            checksum_address(personal_space_admin_plugin, None),
        );

        self.neo4j.find_one(query).await
    }

    pub async fn get_proposal_by_id_and_address(
        &self,
        proposal_id: &str,
        plugin_address: &str,
    ) -> anyhow::Result<Option<models::Proposal>> {
        let query = neo4rs::query(&format!(
            "MATCH (n:`{PROPOSAL_TYPE}` {{onchain_proposal_id: $proposal_id, plugin_address: $plugin_address}}) RETURN n",
            PROPOSAL_TYPE = system_ids::PROPOSAL_TYPE,
        ))
        .param("proposal_id", proposal_id)
        .param("plugin_address", plugin_address);

        self.neo4j.find_one(query).await
    }

    pub async fn add_subspace(
        &self,
        block: &models::BlockMetadata,
        space_id: &str,
        subspace_id: &str,
    ) -> anyhow::Result<()> {
        self.upsert_relation(
            system_ids::INDEXER_SPACE_ID,
            block,
            Relation::new(
                &ids::create_geo_id(),
                subspace_id,
                space_id,
                system_ids::PARENT_SPACE,
                models::ParentSpace,
            ),
        )
        .await?;

        Ok(())
    }

    /// Add an editor to a space
    pub async fn add_editor(
        &self,
        space_id: &str,
        account: &models::GeoAccount,
        editor_relation: &models::SpaceEditor,
        block: &models::BlockMetadata,
    ) -> anyhow::Result<()> {
        self.upsert_node(
            system_ids::INDEXER_SPACE_ID,
            block,
            Node::new(account.id.clone(), account.clone()).with_type(system_ids::GEO_ACCOUNT),
        )
        .await?;

        self.upsert_relation(
            system_ids::INDEXER_SPACE_ID,
            block,
            Relation::new(
                &ids::create_geo_id(),
                &account.id,
                space_id,
                system_ids::EDITOR_RELATION,
                editor_relation,
            ),
        )
        .await?;

        // Add the editor as a member of the space
        self.upsert_relation(
            system_ids::INDEXER_SPACE_ID,
            block,
            Relation::new(
                &ids::create_geo_id(),
                &account.id,
                space_id,
                system_ids::MEMBER_RELATION,
                models::SpaceMember,
            ),
        )
        .await?;

        tracing::info!(
            "Block #{} ({}): Added editor {} to space {}",
            block.block_number,
            block.timestamp,
            account.id,
            space_id
        );

        Ok(())
    }

    pub async fn remove_editor(
        &self,
        editor_id: &str,
        space_id: &str,
        block: &models::BlockMetadata,
    ) -> anyhow::Result<()> {
        const REMOVE_EDITOR_QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (e:`{GEO_ACCOUNT}` {{id: $editor_id}}) -[r:`{EDITOR_RELATION}`]-> (s:`{INDEXED_SPACE}` {{id: $space_id}})
            DELETE r
            SET e.`{UPDATED_AT}` = datetime($updated_at)
            SET e.`{UPDATED_AT_BLOCK}` = $updated_at_block
            "#,
            GEO_ACCOUNT = system_ids::GEO_ACCOUNT,
            EDITOR_RELATION = system_ids::EDITOR_RELATION,
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(REMOVE_EDITOR_QUERY)
            .param("editor_id", editor_id)
            .param("space_id", space_id)
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string());

        self.neo4j.run(query).await?;

        tracing::info!(
            "Block #{} ({}): Removed editor {} from space {}",
            block.block_number,
            block.timestamp,
            editor_id,
            space_id
        );

        Ok(())
    }

    pub async fn add_member(
        &self,
        space_id: &str,
        account: &models::GeoAccount,
        member_relation: &models::SpaceMember,
        block: &models::BlockMetadata,
    ) -> anyhow::Result<()> {
        self.upsert_node(
            system_ids::INDEXER_SPACE_ID,
            block,
            Node::new(account.id.clone(), account.clone()).with_type(system_ids::GEO_ACCOUNT),
        )
        .await?;

        self.upsert_relation(
            system_ids::INDEXER_SPACE_ID,
            block,
            Relation::new(
                &ids::create_geo_id(),
                &account.id,
                space_id,
                system_ids::MEMBER_RELATION,
                member_relation,
            ),
        )
        .await?;

        tracing::info!(
            "Block #{} ({}): Added member {} to space {}",
            block.block_number,
            block.timestamp,
            account.id,
            space_id
        );

        Ok(())
    }

    /// Remove a member from a space
    pub async fn remove_member(
        &self,
        member_id: &str,
        space_id: &str,
        block: &models::BlockMetadata,
    ) -> anyhow::Result<()> {
        const REMOVE_MEMBER_QUERY: &str = const_format::formatcp!(
            r#"
            MATCH (m:`{GEO_ACCOUNT}` {{id: $member_id}}) -[r:`{MEMBER_RELATION}`]-> (s:`{INDEXED_SPACE}` {{id: $space_id}})
            DELETE r
            SET m.`{UPDATED_AT}` = datetime($updated_at)
            SET m.`{UPDATED_AT_BLOCK}` = $updated_at_block
            "#,
            GEO_ACCOUNT = system_ids::GEO_ACCOUNT,
            MEMBER_RELATION = system_ids::MEMBER_RELATION,
            INDEXED_SPACE = system_ids::INDEXED_SPACE,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let query = neo4rs::query(REMOVE_MEMBER_QUERY)
            .param("member_id", member_id)
            .param("space_id", space_id)
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string());

        self.neo4j.run(query).await?;

        tracing::info!(
            "Block #{} ({}): Removed member {} from space {}",
            block.block_number,
            block.timestamp,
            member_id,
            space_id
        );

        Ok(())
    }

    // pub async fn add_vote_cast(
    //     &self,
    //     block: &models::BlockMetadata,
    //     space_id: &str,
    //     account_id: &str,
    //     vote: &models::Vote,
    //     vote_cast: &models::VoteCast,
    // ) -> anyhow::Result<()> {
    //     // self.upsert_relation(
    //     //     INDEXER_SPACE_ID,
    //     //     block,
    //     //     Relation::new(
    //     //         &ids::create_geo_id(),
    //     //         account_id,
    //     //         &vote.id,
    //     //         system_ids::VOTE_CAST_RELATION,
    //     //         vote_cast,
    //     //     ),
    //     // ).await?;
    //     // todo!()

    //     Ok(())
    // }

    // pub async fn add_proposal<T: AsProposal + serde::Serialize>(
    //     &self,
    //     block: &models::BlockMetadata,
    //     space_id: &str,
    //     proposal: &T,
    //     space_proposal_relation: &models::SpaceProposalRelation,
    // ) -> anyhow::Result<()> {
    //     self.upsert_node(
    //         system_ids::INDEXER_SPACE_ID,
    //         block,
    //         Node::new(proposal.as_proposal().id.clone(), proposal)
    //             .with_type(system_ids::PROPOSAL_TYPE)
    //             .with_type(proposal.type_id()),
    //     ).await?;

    //     self.upsert_relation(
    //         system_ids::INDEXER_SPACE_ID,
    //         block,
    //         Relation::new(
    //             &ids::create_geo_id(),
    //             &proposal.as_proposal().id,
    //             space_id,
    //             system_ids::PROPOSAL_SPACE_RELATION,
    //             space_proposal_relation,
    //         ),
    //     ).await?;

    //     Ok(())
    // }

    pub async fn upsert_relation<T: serde::Serialize>(
        &self,
        space_id: &str,
        block: &models::BlockMetadata,
        relation: Relation<T>,
    ) -> anyhow::Result<()> {
        let query_string = format!(
            r#"
            MERGE (from {{id: $from_id}}) -[r:`{relation_type}` {{id: $id}}]-> (to {{id: $to_id}})
            ON CREATE SET r += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET r += {{
                `{SPACE}`: $space_id,
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            SET r += $data
            "#,
            relation_type = relation.relation_type,
            SPACE = system_ids::SPACE,
            CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let bolt_data = match serde_value_to_bolt(serde_json::to_value(&relation.data)?) {
            neo4rs::BoltType::Map(map) => neo4rs::BoltType::Map(map),
            _ => neo4rs::BoltType::Map(Default::default()),
        };

        let query = neo4rs::query(&query_string)
            .param("id", relation.id.clone())
            .param("from_id", relation.from.clone())
            .param("to_id", relation.to.clone())
            .param("space_id", space_id)
            .param("created_at", block.timestamp.to_rfc3339())
            .param("created_at_block", block.block_number.to_string())
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string())
            .param("data", bolt_data);

        self.neo4j.run(query).await?;

        Ok(())
    }

    pub async fn upsert_node<T: serde::Serialize>(
        &self,
        space_id: &str,
        block: &models::BlockMetadata,
        node: Node<T>,
    ) -> anyhow::Result<()> {
        const UPSERT_NODE_QUERY: &str = const_format::formatcp!(
            r#"
            MERGE (n {{id: $id}})
            ON CREATE SET n += {{
                `{CREATED_AT}`: datetime($created_at),
                `{CREATED_AT_BLOCK}`: $created_at_block
            }}
            SET n:$($labels)
            SET n += {{
                `{SPACE}`: $space_id,
                `{UPDATED_AT}`: datetime($updated_at),
                `{UPDATED_AT_BLOCK}`: $updated_at_block
            }}
            SET n += $data
            "#,
            SPACE = system_ids::SPACE,
            CREATED_AT = system_ids::CREATED_AT_TIMESTAMP,
            CREATED_AT_BLOCK = system_ids::CREATED_AT_BLOCK,
            UPDATED_AT = system_ids::UPDATED_AT_TIMESTAMP,
            UPDATED_AT_BLOCK = system_ids::UPDATED_AT_BLOCK,
        );

        let bolt_data = match serde_value_to_bolt(serde_json::to_value(&node.data)?) {
            neo4rs::BoltType::Map(map) => neo4rs::BoltType::Map(map),
            _ => neo4rs::BoltType::Map(Default::default()),
        };

        let query = neo4rs::query(UPSERT_NODE_QUERY)
            .param("id", node.id.clone())
            .param("space_id", space_id)
            .param("created_at", block.timestamp.to_rfc3339())
            .param("created_at_block", block.block_number.to_string())
            .param("updated_at", block.timestamp.to_rfc3339())
            .param("updated_at_block", block.block_number.to_string())
            .param("labels", node.types)
            .param("data", bolt_data);

        self.neo4j.run(query).await?;

        Ok(())
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
            .map(Ok) // Convert to Result to be able to use try_for_each
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
