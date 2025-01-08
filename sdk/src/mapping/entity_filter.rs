use crate::{mapping::ValueType, system_ids};

/// Filter used on toplevel entity queries
#[derive(Clone, Debug, Default)]
pub struct EntityFilter {
    pub id: Option<String>,
    pub space_id: Option<String>,
    pub types_contain: Option<Vec<String>>,
    pub attributes_contain: Option<Vec<EntityAttributeFilter>>,
}

impl EntityFilter {
    pub(crate) fn query(&self) -> neo4rs::Query {
        let query = format!(
            r#"
            {match_clause}
            {where_clause}
            RETURN n
            "#,
            match_clause = self.match_clause(),
            where_clause = self.where_clause(),
        );

        neo4rs::query(&query)
            .param("types", self.types_contain.clone().unwrap_or_default())
            .param("space_id", self.space_id.clone().unwrap_or_default())
    }

    fn match_clause(&self) -> String {
        match self.types_contain.as_ref() {
            Some(_) => {
                format!(
                    r#"
                    MATCH {match_clause_node} <-[:`{FROM_ENTITY}`]- (:`{TYPES}`) -[:`{TO_ENTITY}`]-> (t)
                    "#,
                    FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
                    TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
                    TYPES = system_ids::TYPES,
                    match_clause_node = self.match_clause_node(),
                )
            }
            None => format!(
                "MATCH ({match_clause_node})",
                match_clause_node = self.match_clause_node(),
            ),
        }
    }

    fn match_clause_node(&self) -> String {
        match (self.id.as_ref(), self.space_id.as_ref()) {
            (Some(_), Some(_)) => {
                "(n {id: $id, space_id: $space_id})".to_string()
            }
            (None, Some(_)) => {
                "(n {space_id: $space_id})".to_string()
            }
            (Some(_), None) => {
                "(n {id: $id})".to_string()
            }
            (None, None) => {
                "(n)".to_string()
            }
        }
    }

    fn where_clause(&self) -> String {
        fn _get_attr_query(attrs: &[EntityAttributeFilter]) -> String {
            attrs
                .iter()
                .map(|attr| attr.query())
                .collect::<Vec<_>>()
                .join("\nAND ")
        }

        match (
            self.types_contain.as_ref(),
            self.attributes_contain.as_ref(),
        ) {
            (Some(_), Some(attrs)) => {
                format!(
                    r#"
                    WHERE t.id IN $types
                    AND {}
                    "#,
                    _get_attr_query(attrs)
                )
            }
            (Some(_), None) => "WHERE t.id IN $types".to_string(),
            (None, Some(attrs)) => {
                format!(
                    r#"
                    WHERE {}
                    "#,
                    _get_attr_query(attrs)
                )
            }
            (None, None) => Default::default(),
        }
    }
}

/// Filter used on entity attributes queries (i.e.: when we want to get entities 
/// based on their attributes)
#[derive(Clone, Debug, Default)]
pub struct EntityAttributeFilter {
    pub attribute: String,
    pub value: Option<String>,
    pub value_type: Option<ValueType>,
}

impl EntityAttributeFilter {
    fn query(&self) -> String {
        match self {
            Self {
                attribute,
                value: Some(value),
                value_type: Some(value_type),
            } => {
                format!("n.`{attribute}` = {value} AND n.`{attribute}.type` = {value_type}")
            }
            Self {
                attribute,
                value: Some(value),
                value_type: None,
            } => {
                format!("n.`{attribute}` = {value}")
            }
            Self {
                attribute,
                value: None,
                value_type: Some(value_type),
            } => {
                format!("n.`{attribute}.type` = {value_type}")
            }
            Self {
                attribute,
                value: None,
                value_type: None,
            } => {
                format!("n.`{attribute}` IS NOT NULL")
            }
        }
    }
}

/// Filter used on entity relations queries (i.e.: when we already have an entity and 
/// want to get its relations and related entities)
#[derive(Clone, Debug, Default)]
pub struct EntityRelationFilter {
    pub id: Option<String>,
    pub to_id: Option<String>,
    pub space_id: Option<String>,
    pub relation_type: Option<String>,
}

impl EntityRelationFilter {
    pub fn query(&self, from_id: &str) -> neo4rs::Query {
        let query = format!(
            r#"
            {match_clause}
            RETURN to, r
            "#,
            match_clause = self.match_clause(),
        );

        neo4rs::query(&query)
            .param("id", self.id.clone().unwrap_or_default())
            .param("from_id", from_id)
            .param("to_id", self.to_id.clone().unwrap_or_default())
            .param("space_id", self.space_id.clone().unwrap_or_default())
            .param("relation_type", self.relation_type.clone().unwrap_or_default())
    }

    fn match_clause(&self) -> String {
        format!(
            "MATCH {match_clause_from} <-[:`{FROM_ENTITY}`]- {match_clause_relation} -[:`{TO_ENTITY}`]-> {match_clause_to}",
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            match_clause_from = self.match_clause_from(),
            match_clause_relation = self.match_clause_relation(),
            match_clause_to = self.match_clause_to(),
        )
    }

    fn match_clause_from(&self) -> String {
        match self.space_id.as_ref() {
            Some(_) => 
                "({id: $from_id, space_id: $space_id})".to_string(),
            None =>
                "({id: $from_id})".to_string(),
        }
    }

    fn match_clause_to(&self) -> String {
        match (self.to_id.as_ref(), self.space_id.as_ref()) {
            (Some(_), Some(_)) => 
                "(to {id: $to_id, space_id: $space_id})".to_string(),
            (None, Some(_)) =>
                "(to {space_id: $space_id})".to_string(),
            (Some(_), None) =>
                "(to {id: $to_id})".to_string(),
            (None, None) => "(to)".to_string(),
        }
    }

    fn match_clause_relation(&self) -> String {
        match (self.id.as_ref(), self.relation_type.as_ref()) {
            (Some(_), Some(rel_type)) => 
                format!("(r:`{rel_type}` {{id: $id}})", rel_type = rel_type),
            (None, Some(rel_type)) =>
                format!("(r:`{rel_type}`)", rel_type = rel_type),
            (Some(_), None) =>
                "(r {id: $id})".to_string(),
            (None, None) => "(r)".to_string(),
        }
    }
}