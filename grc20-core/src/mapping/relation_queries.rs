use crate::system_ids;

use super::{query_utils::QueryPart, EntityFilter, PropFilter};

#[derive(Clone, Debug, Default)]
pub struct RelationFilter {
    pub(crate) id: Option<PropFilter<String>>,
    pub(crate) relation_type: Option<EntityFilter>,
    pub(crate) from_: Option<EntityFilter>,
    pub(crate) to_: Option<EntityFilter>,
}

impl RelationFilter {
    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn relation_type(mut self, relation_type: EntityFilter) -> Self {
        self.relation_type = Some(relation_type);
        self
    }

    pub fn from_(mut self, from_: EntityFilter) -> Self {
        self.from_ = Some(from_);
        self
    }

    pub fn to_(mut self, to_: EntityFilter) -> Self {
        self.to_ = Some(to_);
        self
    }

    pub fn into_query_part(self, node_var: &str) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause(format!(
                "({node_var}) -[r_from:`{}`]-> (from:Entity)",
                system_ids::RELATION_FROM_ATTRIBUTE
            ))
            .match_clause(format!(
                "({node_var}) -[r_to:`{}`]-> (to:Entity)",
                system_ids::RELATION_TO_ATTRIBUTE
            ))
            .match_clause(format!(
                "({node_var}) -[r_rt:`{}`]-> (rt:Entity)",
                system_ids::RELATION_TYPE_ATTRIBUTE
            ))
            .match_clause(format!(
                r#"({node_var}) -[r_index:ATTRIBUTE]-> (index:Attribute {{id: "{}"}})"#,
                system_ids::RELATION_INDEX
            ));

        if let Some(id_filter) = self.id {
            query_part.merge_mut(id_filter.into_query_part(node_var, "id"));
        };

        if let Some(relation_type) = self.relation_type {
            query_part = query_part.merge(relation_type.into_query_part("rt"));
        }

        if let Some(from_filter) = self.from_ {
            query_part = query_part.merge(from_filter.into_query_part("from"));
        }

        if let Some(to_filter) = self.to_ {
            query_part = query_part.merge(to_filter.into_query_part("to"));
        }

        query_part
    }
}
