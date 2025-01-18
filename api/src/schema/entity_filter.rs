use juniper::GraphQLInputObject;

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
#[derive(Debug, GraphQLInputObject)]
pub struct EntityFilter {
    pub id: Option<String>,
    // pub space_id: Option<String>,
    pub types_contain: Option<Vec<String>>,
    pub attributes: Option<Vec<EntityAttributeFilter>>,
}

#[derive(Debug, GraphQLInputObject)]
pub struct EntityAttributeFilter {
    pub attribute: String,
    pub value: Option<String>,
    pub value_type: Option<ValueType>,
}

#[derive(Debug, GraphQLInputObject)]
pub struct AttributeFilter {
    pub(crate) value_type: Option<ValueType>,
}

#[derive(Debug, GraphQLInputObject)]
pub struct EntityRelationFilter {
    pub id: Option<String>,
    pub to_id: Option<String>,
    // pub space_id: Option<String>,
    pub relation_type: Option<String>,
}