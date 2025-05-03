use super::{prop_filter::PropFilter, query_builder::MatchQuery, version_filter::VersionFilter};

#[derive(Clone, Debug)]
pub struct AttributeFilter {
    attribute: String,
    space_id: Option<PropFilter<String>>,
    value: Option<PropFilter<String>>,
    value_type: Option<PropFilter<String>>,
    version: VersionFilter,
}

impl AttributeFilter {
    pub fn new(attribute: &str) -> Self {
        Self {
            attribute: attribute.to_owned(),
            space_id: None,
            value: None,
            value_type: None,
            version: VersionFilter::default(),
        }
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn space_id_mut(&mut self, space_id: PropFilter<String>) -> &mut Self {
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

    pub fn version(mut self, space_version: impl Into<String>) -> Self {
        self.version.version_mut(space_version.into());
        self
    }

    pub fn version_opt(mut self, space_version: Option<String>) -> Self {
        self.version.version_opt(space_version);
        self
    }

    pub fn version_mut(&mut self, space_version: impl Into<String>) -> &mut Self {
        self.version.version_mut(space_version.into());
        self
    }

    pub fn subquery(&self, node_var: &str) -> MatchQuery {
        let attr_rel_var = format!("r_{node_var}_{}", self.attribute);
        let attr_node_var = format!("{node_var}_{}", self.attribute);

        MatchQuery::new(
            format!("({node_var}) -[{attr_rel_var}:ATTRIBUTE]-> ({attr_node_var}:Attribute {{id: $attribute}})")
        )
            .r#where(self.version.subquery(&attr_rel_var))
            .where_opt(
                self.space_id.as_ref().map(|space_id| space_id.subquery(&attr_rel_var, "space_id", None))
            )
            .where_opt(
                self.value.as_ref().map(|value| value.subquery(&attr_node_var, "value", None))
            )
            .where_opt(
                self.value_type.as_ref().map(|value_type| value_type.subquery(&attr_node_var, "value_type", None))
            )
            .params("attribute", self.attribute.clone())
    }
}
