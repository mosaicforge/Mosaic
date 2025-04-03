use juniper::GraphQLInputObject;

#[derive(Debug, GraphQLInputObject)]
pub struct TripleFilter {
    pub entity_id: Option<String>,
    pub entity_id_not: Option<String>,
    pub entity_id_in: Option<Vec<String>>,
    pub entity_id_not_in: Option<Vec<String>>,

    pub attribute_id: Option<String>,
    pub attribute_id_not: Option<String>,
    pub attribute_id_in: Option<Vec<String>>,
    pub attribute_id_not_in: Option<Vec<String>>,

    pub space_id: Option<String>,
    pub space_id_not: Option<String>,
    pub space_id_in: Option<Vec<String>>,
    pub space_id_not_in: Option<Vec<String>>,

    pub value: Option<String>,
    pub value_not: Option<String>,
    pub value_in: Option<Vec<String>>,
    pub value_not_in: Option<Vec<String>>,

    pub value_type: Option<String>,
    pub value_type_not: Option<String>,
    pub value_type_in: Option<Vec<String>>,
    pub value_type_not_in: Option<Vec<String>>,
}
