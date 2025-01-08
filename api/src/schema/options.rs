use juniper::GraphQLObject;


#[derive(Debug, GraphQLObject)]
pub struct Options {
    pub(crate) format: Option<String>,
    pub(crate) unit: Option<String>,
    pub(crate) language: Option<String>,
}