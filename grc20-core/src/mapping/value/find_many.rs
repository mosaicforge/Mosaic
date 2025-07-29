use crate::mapping::query_utils::{MatchQuery, PropFilter, QueryBuilder, Subquery};
use uuid::Uuid;

pub fn find_many(neo4j: &neo4rs::Graph) -> FindManyQuery {
    FindManyQuery::new(neo4j)
}

#[derive(Clone)]
pub struct FindManyQuery {
    neo4j: neo4rs::Graph,
    pub property: Option<PropFilter<Uuid>>,
    pub value: Option<PropFilter<String>>,
    pub entity: Option<PropFilter<Uuid>>,
}

impl FindManyQuery {
    pub fn new(neo4j: &neo4rs::Graph) -> Self {
        Self {
            neo4j: neo4j.clone(),
            property: None,
            value: None,
            entity: None,
        }
    }

    pub fn property(mut self, filter: impl Into<PropFilter<Uuid>>) -> Self {
        self.property = Some(filter.into());
        self
    }

    pub fn value(mut self, filter: impl Into<PropFilter<String>>) -> Self {
        self.value = Some(filter.into());
        self
    }

    pub fn entity(mut self, filter: impl Into<PropFilter<Uuid>>) -> Self {
        self.entity = Some(filter.into());
        self
    }

    fn subquery(&self) -> QueryBuilder {
        QueryBuilder::default()
            .subquery(
                MatchQuery::new("(e:Entity) -[r:PROPERTIES]-> (props:Properties)")
                    .where_opt(
                        self.entity
                            .clone()
                            .map(|s| s.as_string_filter().subquery("e", "id", None)),
                    )
                    .where_opt(self.property.clone().map(|s| {
                        s.as_string_filter()
                            .subquery("props", "id", Some("props[$property_key]"))
                    }))
                    .where_opt(
                        self.value
                            .as_ref()
                            .map(|s| s.subquery("n", "value", Some("props[$property_key]"))),
                    ),
            )
            .subquery("RETURN n{.*, entity: e.id}")
    }
}
