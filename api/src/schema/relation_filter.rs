use juniper::GraphQLInputObject;

/// Relation filter input object
#[derive(Debug, GraphQLInputObject)]
pub struct RelationFilter {
    /// Filter by relation types
    pub(crate) relation_type: Option<String>,
}
