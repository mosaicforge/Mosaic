use neo4rs::Query;

use super::{query_utils::{QueryPart, VersionFilter}, EntityFilter, PropFilter};

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

    pub fn into_query_part(self, edge_var: &str, from: &str, to: &str) -> QueryPart {
        let mut query_part = QueryPart::default()
            .match_clause(format!("(rt:Entity {{id: {edge_var}.relation_type}})"));

        if let Some(id_filter) = self.id {
            query_part.merge_mut(id_filter.into_query_part(edge_var, "id", None));
        };

        if let Some(relation_type) = self.relation_type {
            query_part = query_part.merge(relation_type.into_query_part("rt"));
        }

        if let Some(from_filter) = self.from_ {
            query_part = query_part.merge(from_filter.into_query_part(from));
        }

        if let Some(to_filter) = self.to_ {
            query_part = query_part.merge(to_filter.into_query_part(to));
        }

        query_part
    }
}

pub struct MatchOneRelation<'a> {
    id: String,
    space_id: String,
    space_version: &'a VersionFilter,
}

impl<'a> MatchOneRelation<'a> {
    pub fn new(id: String, space_id: impl Into<String>, space_version: &'a VersionFilter) -> Self {
        Self { id, space_id: space_id.into(), space_version }
    }

    pub fn chain(
        self,
        from_node_var: impl Into<String>,
        to_node_var: impl Into<String>,
        edge_var: impl Into<String>,
        next: QueryPart,
    ) -> QueryPart {
        let from_node_var = from_node_var.into();
        let to_node_var = to_node_var.into();
        let edge_var = edge_var.into();

        QueryPart::default()
            .match_clause(format!(
                "({from_node_var}:Entity)-[{edge_var}:RELATION {{id: $id, space_id: $space_id}}]->({to_node_var}:Entity)",
            ))
            .merge(self.space_version.into_query_part(&edge_var))
            .limit(1)
            .order_by_clause(format!("{edge_var}.index"))
            .with_clause(format!("{from_node_var}, {edge_var}, {to_node_var}"), next)
            .params("id", self.id)
            .params("space_id", self.space_id)
    }
}