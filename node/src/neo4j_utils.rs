use futures::TryStreamExt;
use neo4rs::BoltType;
use serde::{Deserialize, Serialize};

/// Extension methods for the Neo4j graph database.
pub trait Neo4jExt {
    /// Find a single node or relationship from the given query and attempt to
    /// deserialize it into the given type.
    fn find_one<T: for<'a> Deserialize<'a> + Send>(
        &self,
        query: neo4rs::Query,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<T>>> + Send;

    /// Find all nodes and/or relationships from the given query and attempt to
    /// deserialize them into the given type.
    /// Note: If the query returns both nodes and relations, neo4j will group the results
    /// in tuples of (node, relation). For example: `MATCH (n) -[r]-> () RETURN n, r` will
    /// return a list of `{"n": ..., "r": ...}` JSON objects.
    fn find_all<T: for<'a> Deserialize<'a> + Send>(
        &self,
        query: neo4rs::Query,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<T>>> + Send;

    fn insert_one<T: Serialize + Send>(
        &self,
        node: T,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn insert_many<T: Serialize>(
        &self,
        node: Vec<T>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>>;
}

impl Neo4jExt for neo4rs::Graph {
    async fn find_one<T: for<'a> Deserialize<'a>>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Option<T>> {
        Ok(self
            .execute(query)
            .await?
            .next()
            .await?
            .map(|row| row.to())
            .transpose()?)
    }

    async fn find_all<T: for<'a> Deserialize<'a>>(
        &self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Vec<T>> {
        Ok(self
            .execute(query)
            .await?
            .into_stream_as::<T>()
            .try_collect::<Vec<_>>()
            .await?)
    }

    async fn insert_one<T: Serialize>(&self, node: T) -> anyhow::Result<()> {
        let json = serde_json::to_value(&node)?;

        let label = json.get("$type").and_then(|value| value.as_str());

        let query = if let Some(label) = label {
            neo4rs::query(&format!("CREATE (n:{label}) SET n = $node"))
        } else {
            neo4rs::query("CREATE (n) SET n = $node")
        };

        let query = query.param("node", serde_value_to_bolt(serde_json::to_value(&node)?));

        self.run(query).await?;

        Ok(())
    }

    async fn insert_many<T: Serialize>(&self, node: Vec<T>) -> anyhow::Result<()> {
        let json = serde_json::to_value(&node)?;

        let label = json.get("$type").and_then(|value| value.as_str());

        let query = if let Some(label) = label {
            neo4rs::query(&format!(
                "UNWIND $nodes AS node CREATE (n:{label}) SET n = node"
            ))
        } else {
            neo4rs::query("UNWIND $nodes AS node CREATE (n) SET n = node")
        };

        let query = query.param("nodes", serde_value_to_bolt(json));

        self.run(query).await?;

        Ok(())
    }
}

/// Extension methods for Neo4j graph database transactions.
pub trait Neo4jMutExt {
    /// Find a single node or relationship from the given query and attempt to
    /// deserialize it into the given type.
    fn find_one<T: for<'a> Deserialize<'a> + Send>(
        &mut self,
        query: neo4rs::Query,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<T>>> + Send;

    /// Find all nodes and/or relationships from the given query and attempt to
    /// deserialize them into the given type.
    /// Note: If the query returns both nodes and relations, neo4j will group the results
    /// in tuples of (node, relation). For example: `MATCH (n) -[r]-> () RETURN n, r` will
    /// return a list of `{"n": ..., "r": ...}` JSON objects.
    fn find_all<T: for<'a> Deserialize<'a> + Send>(
        &mut self,
        query: neo4rs::Query,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<T>>> + Send;

    fn insert_one<T: Serialize + Send>(
        &mut self,
        node: T,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    fn insert_many<T: Serialize>(
        &mut self,
        node: Vec<T>,
    ) -> impl std::future::Future<Output = anyhow::Result<()>>;
}

impl Neo4jMutExt for neo4rs::Txn {
    async fn find_one<T: for<'a> Deserialize<'a>>(
        &mut self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Option<T>> {
        Ok(self
            .execute(query)
            .await?
            .next(self.handle())
            .await?
            .map(|row| row.to())
            .transpose()?)
    }

    async fn find_all<T: for<'a> Deserialize<'a>>(
        &mut self,
        query: neo4rs::Query,
    ) -> anyhow::Result<Vec<T>> {
        Ok(self
            .execute(query)
            .await?
            .into_stream_as::<T>(self)
            .try_collect::<Vec<_>>()
            .await?)
    }

    async fn insert_one<T: Serialize>(&mut self, node: T) -> anyhow::Result<()> {
        let json = serde_json::to_value(&node)?;

        let label = json.get("$type").and_then(|value| value.as_str());

        let query = if let Some(label) = label {
            neo4rs::query(&format!("CREATE (n:{label}) SET n = $node"))
        } else {
            neo4rs::query("CREATE (n) SET n = $node")
        };

        let query = query.param("node", serde_value_to_bolt(serde_json::to_value(&node)?));

        self.run(query).await?;

        Ok(())
    }

    async fn insert_many<T: Serialize>(&mut self, node: Vec<T>) -> anyhow::Result<()> {
        let json = serde_json::to_value(&node)?;

        let label = json.get("$type").and_then(|value| value.as_str());

        let query = if let Some(label) = label {
            neo4rs::query(&format!(
                "UNWIND $nodes AS node CREATE (n:{label}) SET n = node"
            ))
        } else {
            neo4rs::query("UNWIND $nodes AS node CREATE (n) SET n = node")
        };

        let query = query.param("nodes", serde_value_to_bolt(json));

        self.run(query).await?;

        Ok(())
    }
}

pub fn serde_value_to_bolt(value: serde_json::Value) -> BoltType {
    match value {
        serde_json::Value::Null => BoltType::Null(neo4rs::BoltNull),
        serde_json::Value::Bool(value) => BoltType::Boolean(neo4rs::BoltBoolean { value }),
        serde_json::Value::Number(number) if number.is_i64() => {
            BoltType::Integer(neo4rs::BoltInteger {
                value: number.as_i64().expect("number should be an i64"),
            })
        }
        serde_json::Value::Number(number) if number.is_u64() => {
            BoltType::Integer(neo4rs::BoltInteger {
                value: number.as_u64().expect("number should be a u64") as i64,
            })
        }
        serde_json::Value::Number(number) => BoltType::Float(neo4rs::BoltFloat {
            value: number.as_f64().expect("number should be an f64"),
        }),
        serde_json::Value::String(value) => BoltType::String(neo4rs::BoltString { value }),
        serde_json::Value::Array(vec) => {
            let values = vec.into_iter().map(serde_value_to_bolt).collect();
            BoltType::List(neo4rs::BoltList { value: values })
        }
        serde_json::Value::Object(map) => {
            let properties = map
                .into_iter()
                .filter(|(key, _)| key != "$type")
                .map(|(key, value)| {
                    (
                        neo4rs::BoltString { value: key },
                        serde_value_to_bolt(value),
                    )
                })
                .collect();
            BoltType::Map(neo4rs::BoltMap { value: properties })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_one_node() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let query = neo4rs::query("MATCH (n) RETURN n LIMIT 1");

        let node: Option<serde_json::Value> = graph.find_one(query).await.unwrap();
        println!("{:?}", node);
    }

    #[tokio::test]
    async fn test_find_one_relation() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let query = neo4rs::query("MATCH () -[r]-> () RETURN r LIMIT 1");

        let node: Option<serde_json::Value> = graph.find_one(query).await.unwrap();
        println!("{:?}", node);
    }

    #[tokio::test]
    async fn test_find_all_nodes() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let query = neo4rs::query("MATCH (n:`d7ab40920ab5441e88c35c27952de773`) RETURN n LIMIT 4");

        let nodes: Vec<serde_json::Value> = graph.find_all(query).await.unwrap();
        println!("{:?}", nodes);
    }

    #[tokio::test]
    async fn test_find_all_relations() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let query =
            neo4rs::query("MATCH () -[r:`01412f8381894ab1836565c7fd358cc1`]-> () RETURN r LIMIT 4");

        let nodes: Vec<serde_json::Value> = graph.find_all(query).await.unwrap();
        println!("{:?}", nodes);
    }

    #[tokio::test]
    async fn test_find_all_relations_and_nodes() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let query = neo4rs::query("MATCH () -[r:`577bd9fbb29e4e2bb5f8f48aedbd26ac`]-> () MATCH (n:`3d0b19e0313843479687a351223efcc3`) RETURN n, r LIMIT 8");

        let nodes: Vec<serde_json::Value> = graph.find_all(query).await.unwrap();
        println!("{}", serde_json::to_string_pretty(&nodes).unwrap());
    }

    #[tokio::test]
    async fn test_find_one_node_txn() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let mut txn = graph.start_txn().await.unwrap();

        txn.run(neo4rs::query("CREATE (n:TestNode {name: \"potato\"})"))
            .await
            .unwrap();

        let query = neo4rs::query("MATCH (n:TestNode) RETURN n LIMIT 1");

        let node: Option<serde_json::Value> = txn.find_one(query).await.unwrap();

        txn.rollback().await.unwrap();

        println!("{:?}", node);
    }

    #[tokio::test]
    async fn test_insert_one_txn() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let mut txn = graph.start_txn().await.unwrap();

        #[derive(Deserialize, Debug, Serialize)]
        #[serde(tag = "$type")]
        struct Foo {
            name: String,
        }

        let node = Foo {
            name: "potato".to_string(),
        };

        txn.insert_one(node).await.unwrap();

        let node: Option<Foo> = txn
            .find_one(neo4rs::query("MATCH (n:Foo) RETURN n LIMIT 1"))
            .await
            .unwrap();

        txn.rollback().await.unwrap();

        println!("{:?}", node);
    }

    #[tokio::test]
    async fn test_insert_many_txn() {
        let graph = neo4rs::Graph::new("neo4j://localhost:7687", "", "")
            .await
            .unwrap();

        let mut txn = graph.start_txn().await.unwrap();

        #[derive(Deserialize, Debug, Serialize)]
        #[serde(tag = "$type")]
        struct Foo {
            name: String,
        }

        let nodes = vec![
            Foo {
                name: "potato".to_string(),
            },
            Foo {
                name: "pizza".to_string(),
            },
        ];

        txn.insert_many(nodes).await.unwrap();

        let node: Vec<Foo> = txn
            .find_all(neo4rs::query("MATCH (n:Foo) RETURN n"))
            .await
            .unwrap();

        txn.rollback().await.unwrap();

        println!("{:?}", node);
    }
}
