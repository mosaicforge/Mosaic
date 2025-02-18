use super::{PropFilter, QueryPart, VersionFilter};

#[derive(Clone, Debug, Default)]
pub struct EdgeFilter {
    space_id: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,
}

impl EdgeFilter {
    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn to_id(mut self, to_id: PropFilter<String>) -> Self {
        self.to_id = Some(to_id);
        self
    }

    pub(crate) fn into_query_part(
        self,
        node_var: impl Into<String>,
        r#type: impl Into<String>,
        version: Option<String>,
    ) -> QueryPart {
        let node_var = node_var.into();
        let r#type = r#type.into();

        let mut query = QueryPart::default().match_clause(format!(
            "({node_var}) -[r_{type}:`{type}`]- (r_{type}_to:Entity)"
        ));

        if let Some(space_id) = self.space_id {
            query = query.merge(space_id.into_query_part(&format!("r_{type}"), "space_id"));
        }

        if let Some(to_id) = self.to_id {
            query = query.merge(to_id.into_query_part(&format!("r_{type}_to"), "id"));
        }

        query.merge(VersionFilter::new(version).into_query_part(&format!("r_{type}")))
    }
}
