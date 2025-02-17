use super::{prop_filter::PropFilter, query_part::QueryPart, version_filter::VersionFilter};

#[derive(Clone, Debug)]
pub struct AttributeFilter {
    attribute: String,
    space_id: Option<PropFilter<String>>,
    value: Option<PropFilter<String>>,
    value_type: Option<PropFilter<String>>,
    space_version: VersionFilter,
}

impl AttributeFilter {
    pub fn new(attribute: &str) -> Self {
        Self {
            attribute: attribute.to_owned(),
            space_id: None,
            value: None,
            value_type: None,
            space_version: VersionFilter::default(),
        }
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn value(mut self, value: PropFilter<String>) -> Self {
        self.value = Some(value);
        self
    }

    pub fn value_type(mut self, value_type: PropFilter<String>) -> Self {
        self.value_type = Some(value_type);
        self
    }

    pub fn space_version(mut self, space_version: impl Into<String>) -> Self {
        self.space_version.version_mut(space_version.into());
        self
    }

    pub fn into_query_part(self, node_var: &str) -> QueryPart {
        let attr_rel_var = format!("r_{node_var}_{}", self.attribute);
        let attr_node_var = format!("{node_var}_{}", self.attribute);

        let mut query_part = QueryPart::default()
            .match_clause(format!("({node_var}) -[{attr_rel_var}:ATTRIBUTE]-> ({attr_node_var} {{attribute: $attribute}})"))
            .params("attribute", self.attribute)
            .merge(self.space_version.into_query_part(&attr_rel_var));

        if let Some(space_id) = self.space_id {
            query_part.merge_mut(space_id.into_query_part(&attr_rel_var, "space_id"));
        }

        if let Some(value) = self.value {
            query_part.merge_mut(value.into_query_part(&attr_node_var, "value"));
        }

        if let Some(value_type) = self.value_type {
            query_part.merge_mut(value_type.into_query_part(&attr_node_var, "value_type"));
        }

        query_part
    }
}
