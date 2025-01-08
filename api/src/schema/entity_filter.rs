use juniper::GraphQLInputObject;
use sdk::mapping;

use crate::schema::ValueType;

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
///
#[derive(Debug, GraphQLInputObject)]
pub struct EntityFilter {
    id: Option<String>,
    space_id: Option<String>,
    types_contain: Option<Vec<String>>,
    attributes_contain: Option<Vec<EntityAttributeFilter>>,
}

impl From<EntityFilter> for mapping::EntityFilter {
    fn from(filter: EntityFilter) -> Self {
        mapping::EntityFilter {
            id: filter.id,
            space_id: filter.space_id,
            types_contain: filter.types_contain,
            attributes_contain: filter
                .attributes_contain
                .map(|filters| filters.into_iter().map(Into::into).collect()),
        }
    }
}

#[derive(Debug, GraphQLInputObject)]
pub struct EntityAttributeFilter {
    attribute: String,
    value: Option<String>,
    value_type: Option<ValueType>,
}

impl From<EntityAttributeFilter> for mapping::EntityAttributeFilter {
    fn from(filter: EntityAttributeFilter) -> Self {
        mapping::EntityAttributeFilter {
            attribute: filter.attribute,
            value: filter.value,
            value_type: filter.value_type.map(Into::into),
        }
    }
}

#[derive(Debug, GraphQLInputObject)]
pub struct AttributeFilter {
    pub(crate) value_type: Option<ValueType>,
}

#[derive(Debug, GraphQLInputObject)]
pub struct EntityRelationFilter {
    pub id: Option<String>,
    pub to_id: Option<String>,
    pub space_id: Option<String>,
    pub relation_type: Option<String>,
}

impl From<EntityRelationFilter> for mapping::EntityRelationFilter {
    fn from(value: EntityRelationFilter) -> Self {
        mapping::EntityRelationFilter {
            id: value.id,
            to_id: value.to_id,
            space_id: value.space_id,
            relation_type: value.relation_type,
        }
    }
}
