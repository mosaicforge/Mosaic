use crate::{mapping::ValueType, system_ids};

pub struct EntityWhereFilter {
    pub id: Option<String>,
    pub space_id: Option<String>,
    pub types_contain: Option<Vec<String>>,
    pub attributes_contain: Option<Vec<EntityAttributeFilter>>,
}

impl EntityWhereFilter {
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
                    MATCH (n {match_clause_attrs}) <-[:`{FROM_ENTITY}`]- (:`{TYPES}`) -[:`{TO_ENTITY}`]-> (t)
                    "#,
                    FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
                    TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
                    TYPES = system_ids::TYPES,
                    match_clause_attrs = self.match_clause_attrs(),
                )
            }
            None => format!(
                "MATCH (n {match_clause_attrs})",
                match_clause_attrs = self.match_clause_attrs(),
            ),
        }
    }

    fn match_clause_attrs(&self) -> String {
        match (self.id.as_ref(), self.space_id.as_ref()) {
            (Some(_), Some(_)) => {
                "{id: $id, space_id: $space_id}".to_string()
            }
            (None, Some(_)) => {
                "{space_id: $space_id}".to_string()
            }
            (Some(_), None) => {
                "{id: $id}".to_string()
            }
            (None, None) => {
                "".to_string()
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

#[derive(Default)]
pub struct EntityWhereFilterBuilder {
    pub id: Option<String>,
    pub space_id: Option<String>,
    pub types_contain: Option<Vec<String>>,
    pub attributes_contain: Option<Vec<EntityAttributeFilter>>,
}

impl EntityWhereFilterBuilder {
    pub fn build(self) -> EntityWhereFilter {
        EntityWhereFilter {
            id: self.id,
            space_id: self.space_id,
            types_contain: self.types_contain,
            attributes_contain: self.attributes_contain,
        }
    }

    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn space_id(mut self, space_id: String) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn types_contain(mut self, r#type: String) -> Self {
        if let Some(mut types) = self.types_contain {
            types.push(r#type);
            self.types_contain = Some(types);
        } else {
            self.types_contain = Some(vec![r#type]);
        }
        self
    }

    pub fn set_types_contain(mut self, types: Vec<String>) -> Self {
        self.types_contain = Some(types);
        self
    }

    pub fn attributes_contain(mut self, attribute: EntityAttributeFilter) -> Self {
        if let Some(mut attributes) = self.attributes_contain {
            attributes.push(attribute);
            self.attributes_contain = Some(attributes);
        } else {
            self.attributes_contain = Some(vec![attribute]);
        }
        self
    }

    pub fn set_attributes_contain(mut self, attributes: Vec<EntityAttributeFilter>) -> Self {
        self.attributes_contain = Some(attributes);
        self
    }
}

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