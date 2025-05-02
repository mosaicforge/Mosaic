use crate::mapping::{
    query_utils::{QueryPart, VersionFilter},
    EntityFilter, PropFilter,
};

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

    pub fn compile(&self, edge_var: &str, from: &str, to: &str) -> QueryPart {
        QueryPart::default()
            .match_clause(format!("(rt:Entity {{id: {edge_var}.relation_type}})"))
            .merge_opt(self.id.as_ref().map(|id| id.compile(edge_var, "id", None)))
            .merge_opt(self.relation_type.as_ref().map(|rt| rt.compile("rt")))
            .merge_opt(
                self.from_
                    .as_ref()
                    .map(|from_filter| from_filter.compile(from)),
            )
            .merge_opt(self.to_.as_ref().map(|to_filter| to_filter.compile(to)))
    }
}

pub struct MatchOneRelation<'a> {
    id: String,
    space_id: String,
    space_version: &'a VersionFilter,
}

impl<'a> MatchOneRelation<'a> {
    pub fn new(id: String, space_id: impl Into<String>, space_version: &'a VersionFilter) -> Self {
        Self {
            id,
            space_id: space_id.into(),
            space_version,
        }
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
            .merge(self.space_version.compile(&edge_var))
            .limit(1)
            .order_by_clause(format!("{edge_var}.index"))
            .with_clause(format!("{from_node_var}, {edge_var}, {to_node_var}"), next)
            .params("id", self.id)
            .params("space_id", self.space_id)
    }
}
