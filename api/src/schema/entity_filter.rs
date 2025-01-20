use juniper::GraphQLInputObject;

use sdk::mapping;

use crate::schema::{EntityAttributeFilter, ValueType};

/// Entity filter input object
///
/// ```graphql
/// query {
///     entities(where: {
///         space_id: "BJqiLPcSgfF8FRxkFr76Uy",
///         types_contain: ["XG26vy98XAA6cR6DosTALk", "XG26vy98XAA6cR6DosTALk"],
///         attributes_contain: [
///             {id: "XG26vy98XAA6cR6DosTALk", value: "value", value_type: TEXT},
///         ]
///     })
/// }
/// ```
#[derive(Debug, GraphQLInputObject)]
pub struct EntityFilter {
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    // pub space_id: Option<String>,

    /// Exact match for the entity types
    pub types: Option<Vec<String>>,
    pub types_not: Option<Vec<String>>,
    pub types_contains: Option<Vec<String>>,
    pub types_not_contains: Option<Vec<String>>,

    pub attributes: Option<Vec<EntityAttributeFilter>>,
}

impl EntityFilter {
    pub fn add_to_entity_query(self, mut query: mapping::entity_queries::FindMany) -> mapping::entity_queries::FindMany {
        if let Some(id) = self.id {
            query = query.id(&id);
        }

        if let Some(id_not) = self.id_not {
            query = query.id_not(&id_not);
        }

        if let Some(id_in) = self.id_in {
            query = query.id_in(id_in);
        }

        if let Some(id_not_in) = self.id_not_in {
            query = query.id_not_in(id_not_in);
        }

        if let Some(types) = self.types {
            query = query.types(types);
        }

        if let Some(types_not) = self.types_not {
            query = query.types_not(types_not);
        }

        if let Some(types_contains) = self.types_contains {
            query = query.types_contains(types_contains);
        }

        if let Some(types_not_contains) = self.types_not_contains {
            query = query.types_not_contains(types_not_contains);
        }

        if let Some(attributes) = self.attributes {
            for attr in attributes {
                query = attr.add_to_entity_query(query);
            }
        }

        query
    }
}

#[derive(Debug, GraphQLInputObject)]
pub struct AttributeFilter {
    pub(crate) value_type: Option<ValueType>,
}

/// Filters the outgoing relations of the entity
#[derive(Debug, GraphQLInputObject)]
pub struct EntityRelationFilter {
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    pub to_id: Option<String>,
    pub to_id_not: Option<String>,
    pub to_id_in: Option<Vec<String>>,
    pub to_id_not_in: Option<Vec<String>>,

    pub relation_type: Option<String>,
    pub relation_type_not: Option<String>,
    pub relation_type_in: Option<Vec<String>>,
    pub relation_type_not_in: Option<Vec<String>>,

    /// Filter the relations by the entity they point to
    pub to_: Option<EntityFilter>,

    pub attributes: Option<Vec<EntityAttributeFilter>>,
}