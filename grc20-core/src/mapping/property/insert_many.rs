use super::models::Property;

pub fn insert_many(neo4j: neo4rs::Graph) -> InsertManyQuery {
    InsertManyQuery::new(neo4j)
}

/// Query struct for inserting multiple properties into the database.
#[derive(Clone)]
pub struct InsertManyQuery {
    /// The properties to be inserted.
    pub properties: Vec<Property>,
    neo4j: neo4rs::Graph,
}

impl InsertManyQuery {
    /// Creates a new InsertManyQuery with an empty property list.
    pub fn new(neo4j: neo4rs::Graph) -> Self {
        InsertManyQuery {
            properties: Vec::new(),
            neo4j,
        }
    }

    /// Sets the properties to be inserted.
    pub fn properties(mut self, props: Vec<Property>) -> Self {
        self.properties = props;
        self
    }

    /// Adds a property to the query.
    pub fn property(mut self, prop: Property) -> Self {
        self.properties.push(prop);
        self
    }

    /// Executes the query to insert all properties into the Neo4j database.
    pub async fn send(self) -> Result<(), neo4rs::Error> {
        // Use UNWIND for batch insertion
        let query = r#"
            UNWIND $props AS prop
            CREATE (p:Entity {id: prop.id, data_type: prop.data_type})
        "#;

        // Convert Vec<Property> to Vec<BoltType> for Neo4j parameter
        let bolt_props: Vec<neo4rs::BoltType> =
            self.properties.into_iter().map(Into::into).collect();

        self.neo4j
            .run(neo4rs::query(query).param("props", bolt_props))
            .await?;

        Ok(())
    }
}
