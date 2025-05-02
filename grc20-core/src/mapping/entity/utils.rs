use rand::distributions::DistString;

use crate::{
    mapping::{
        query_utils::{QueryPart, VersionFilter},
        AttributeFilter, PropFilter,
    },
    system_ids,
};

#[derive(Clone, Debug, Default)]
pub struct EntityFilter {
    pub(crate) id: Option<PropFilter<String>>,
    pub(crate) attributes: Vec<AttributeFilter>,
    pub(crate) relations: Option<EntityRelationFilter>,
    pub(crate) space_id: Option<PropFilter<String>>,
}

impl EntityFilter {
    pub fn id(mut self, id: PropFilter<String>) -> Self {
        self.id = Some(id);
        self
    }

    pub fn attribute(mut self, attribute: AttributeFilter) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn attribute_mut(&mut self, attribute: AttributeFilter) {
        self.attributes.push(attribute);
    }

    pub fn attributes(mut self, attributes: impl IntoIterator<Item = AttributeFilter>) -> Self {
        self.attributes.extend(attributes);
        self
    }

    pub fn attributes_mut(&mut self, attributes: impl IntoIterator<Item = AttributeFilter>) {
        self.attributes.extend(attributes);
    }

    pub fn relations(mut self, relations: impl Into<EntityRelationFilter>) -> Self {
        self.relations = Some(relations.into());
        self
    }

    /// Applies a global space_id to all sub-filters (i.e.: attribute and relation filters).
    /// If a space_id is already set in a sub-filter, it will be overwritten.
    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id.clone());
        self
    }

    pub(crate) fn compile(&self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let mut query_part = QueryPart::default();

        if let Some(id) = &self.id {
            query_part.merge_mut(id.compile(&node_var, "id", None));
        }

        if self.attributes.is_empty() {
            if let Some(space_id) = &self.space_id {
                query_part = query_part
                    .match_clause(format!("({node_var}) -[attribute:ATTRIBUTE]- (:Attribute)",))
                    .merge(space_id.clone().compile("attribute", "space_id", None));
            }
        } else {
            for mut attribute in self.attributes.clone() {
                if let Some(space_id) = &self.space_id {
                    attribute = attribute.space_id(space_id.clone());
                }
                query_part.merge_mut(attribute.into_query_part(&node_var));
            }
        }

        if let Some(mut relations) = self.relations.clone() {
            if let Some(space_id) = &self.space_id {
                relations = relations.space_id(space_id.clone());
            }
            query_part.merge_mut(relations.into_query_part(node_var));
        }

        query_part
    }
}

/// Filter used to:
/// - Filter the relations outgoing from the entity
/// - Filter an entity by its outgoing relations
#[derive(Clone, Debug, Default)]
pub struct EntityRelationFilter {
    relation_type: Option<PropFilter<String>>,
    to_id: Option<PropFilter<String>>,
    space_id: Option<PropFilter<String>>,
    version: VersionFilter,
}

impl EntityRelationFilter {
    pub fn relation_type(mut self, relation_type: impl Into<PropFilter<String>>) -> Self {
        self.relation_type = Some(relation_type.into());
        self
    }

    pub fn to_id(mut self, to_id: impl Into<PropFilter<String>>) -> Self {
        self.to_id = Some(to_id.into());
        self
    }

    pub fn space_id(mut self, space_id: impl Into<PropFilter<String>>) -> Self {
        self.space_id = Some(space_id.into());
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version.version_mut(version.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type.is_none() && self.to_id.is_none()
    }

    // /// Applies a global space_id to all sub-filters (i.e.: relation_type and to_id filters).
    // /// If a space_id is already set in a sub-filter, it will be overwritten.
    // pub fn with_space_id(mut self, space_id: PropFilter<String>) -> Self {
    //     self.relation_type = self
    //         .relation_type
    //         .map(|filter| filter.space_id(space_id.clone()));

    //     self.to_id = self.to_id.map(|filter| filter.space_id(space_id));

    //     self
    // }

    pub(crate) fn into_query_part(self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let random_suffix: String =
            rand::distributions::Alphanumeric.sample_string(&mut rand::thread_rng(), 4);
        let rel_edge_var = format!("r_{node_var}_{}", random_suffix);
        let to_node_var = format!("r_{node_var}_to");
        let mut query_part = QueryPart::default();

        if !self.is_empty() {
            query_part = query_part
                .match_clause(format!(
                    "({node_var}) -[{rel_edge_var}:RELATION]-> ({to_node_var})",
                ))
                .merge(self.version.compile(&rel_edge_var));

            if let Some(relation_type) = self.relation_type {
                query_part =
                    query_part.merge(relation_type.compile(&rel_edge_var, "relation_type", None));
            }

            if let Some(to_id) = self.to_id {
                query_part = query_part.merge(to_id.compile(&to_node_var, "id", None));
            }

            if let Some(space_id) = self.space_id {
                query_part = query_part.merge(space_id.compile(&rel_edge_var, "space_id", None));
            }
        }

        query_part
    }
}

#[derive(Clone, Debug)]
pub struct MatchEntityAttributes<'a> {
    space_id: &'a Option<PropFilter<String>>,
    version: &'a VersionFilter,
}

impl<'a> MatchEntityAttributes<'a> {
    pub(crate) fn new(
        space_id: &'a Option<PropFilter<String>>,
        version: &'a VersionFilter,
    ) -> Self {
        Self { space_id, version }
    }

    pub fn compile(
        self,
        node_var: impl Into<String>,
        attributes_node_var: impl Into<String>,
    ) -> QueryPart {
        let node_var = node_var.into();
        let attrs_node_var = attributes_node_var.into();

        QueryPart::default()
            .match_clause(format!(
                "({node_var}) -[attribute:ATTRIBUTE]- ({attrs_node_var}:Attribute)"
            ))
            .merge(self.version.compile("attribute"))
            .merge_opt(
                self.space_id
                    .as_ref()
                    .map(|space_id| space_id.compile("attribute", "space_id", None)),
            )
    }

    /// Returns a query part that selects the attributes of an entity `node_var`.
    /// The attributes can be referenced by the `attributes_node_var` variable.
    pub fn chain(
        self,
        node_var: impl Into<String>,
        attributes_node_var: impl Into<String>,
        next: QueryPart,
    ) -> QueryPart {
        let node_var = node_var.into();
        let attrs_node_var = attributes_node_var.into();

        self.compile(&node_var, &attrs_node_var).with_clause(
            format!("{node_var}, {attrs_node_var}{{.*}} AS {attrs_node_var}"),
            next,
        )
    }
}

#[derive(Clone, Debug)]
pub struct MatchEntityTypes<'a> {
    space_id: &'a Option<PropFilter<String>>,
    version: &'a VersionFilter,
}

impl<'a> MatchEntityTypes<'a> {
    pub(crate) fn new(
        space_id: &'a Option<PropFilter<String>>,
        version: &'a VersionFilter,
    ) -> Self {
        Self { space_id, version }
    }

    /// Returns a query part that selects the types of an entity `node_var`.
    /// The types can be referenced by the `types_node_var` variable.
    pub fn compile(
        self,
        node_var: impl Into<String>,
        types_node_var: impl Into<String>,
    ) -> QueryPart {
        let node_var = node_var.into();
        let types_rel_var = format!("r_{node_var}_types");
        let types_node_var = types_node_var.into();

        QueryPart::default()
            .match_clause(format!(r#"({node_var}) -[{types_rel_var}:RELATION {{relation_type: "{}"}}]-> ({types_node_var}:Entity)"#, system_ids::TYPES_ATTRIBUTE))
            .merge(self.version.compile(&types_rel_var))
            .merge_opt(self.space_id.as_ref()
                .map(|space_id| space_id.compile(&types_rel_var, "space_id", None)))
    }

    /// Returns a query part that selects the types of an entity `node_var`.
    /// The types can be referenced by the `types_node_var` variable.
    pub fn chain(
        self,
        node_var: impl Into<String>,
        types_node_var: impl Into<String>,
        next: QueryPart,
    ) -> QueryPart {
        let node_var = node_var.into();
        let types_node_var = types_node_var.into();

        self.compile(&node_var, &types_node_var).with_clause(
            format!("{node_var}, {types_node_var}{{.*}} AS {types_node_var}"),
            next,
        )
    }
}

#[derive(Clone, Debug)]
pub struct MatchEntity<'a> {
    match_attributes: MatchEntityAttributes<'a>,
    match_types: MatchEntityTypes<'a>,
}

impl<'a> MatchEntity<'a> {
    pub(crate) fn new(
        space_id: &'a Option<PropFilter<String>>,
        version: &'a VersionFilter,
    ) -> Self {
        Self {
            match_attributes: MatchEntityAttributes::new(space_id, version),
            match_types: MatchEntityTypes::new(space_id, version),
        }
    }

    /// Returns a query part that selects the entity `node_var` with its
    /// attributes and types.
    /// The query part will end with a `WITH` clause that contains the entity data and
    /// the attributes and types.
    /// The data can be referenced in subsequent queries with the `entity_node_var` variable.
    pub fn chain(
        self,
        node_var: impl Into<String>,
        attributes_node_var: impl Into<String>,
        types_node_var: impl Into<String>,
        next: QueryPart,
    ) -> QueryPart {
        let node_var = node_var.into();
        let attributes_node_var = attributes_node_var.into();
        let types_node_var = types_node_var.into();
        // let entity_node_var = entity_node_var.into();

        // let attributes_node_var = format!("{entity_node_var}_attributes");
        // let types_node_var = format!("{entity_node_var}_types");

        QueryPart::default()
            .merge(self.match_attributes.compile(&node_var, &attributes_node_var))
            .merge(self.match_types.compile(&node_var, &types_node_var))
            .with_clause(format!(
                "{node_var}, collect({attributes_node_var}{{.*}}) AS {attributes_node_var}, collect({types_node_var}{{.*}}) AS {types_node_var}"
            ), next)
    }
}
