use crate::system_ids;

use super::{query_utils::{EdgeFilter, QueryPart}, AttributeFilter, PropFilter};

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

    pub(crate) fn into_query_part(self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let mut query_part = QueryPart::default();

        if let Some(id) = self.id {
            query_part.merge_mut(id.into_query_part(&node_var, "id"));
        }

        if self.attributes.is_empty() {
            if let Some(space_id) = &self.space_id {
                query_part = query_part
                    .match_clause(format!("({node_var}) -[attribute:ATTRIBUTE]- (:Attribute)",))
                    .merge(space_id.clone().into_query_part("attribute", "space_id"));
            }
        } else {
            for mut attribute in self.attributes {
                if let Some(space_id) = &self.space_id {
                    attribute = attribute.space_id(space_id.clone());
                }
                query_part.merge_mut(attribute.into_query_part(&node_var));
            }
        }

        if let Some(mut relations) = self.relations {
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
    relation_type: Option<EdgeFilter>,
    to_id: Option<EdgeFilter>,
    space_id: Option<PropFilter<String>>,
    space_version: Option<String>,
}

impl EntityRelationFilter {
    pub fn relation_type(mut self, relation_type: EdgeFilter) -> Self {
        self.relation_type = Some(relation_type);
        self
    }

    pub fn to_id(mut self, to_id: EdgeFilter) -> Self {
        self.to_id = Some(to_id);
        self
    }

    pub fn space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.space_id = Some(space_id);
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.space_version = Some(version.into());
        self
    }

    pub fn is_empty(&self) -> bool {
        self.relation_type.is_none() && self.to_id.is_none()
    }

    /// Applies a global space_id to all sub-filters (i.e.: relation_type and to_id filters).
    /// If a space_id is already set in a sub-filter, it will be overwritten.
    pub fn with_space_id(mut self, space_id: PropFilter<String>) -> Self {
        self.relation_type = self
            .relation_type
            .map(|filter| filter.space_id(space_id.clone()));

        self.to_id = self.to_id.map(|filter| filter.space_id(space_id));

        self
    }

    pub(crate) fn into_query_part(self, node_var: impl Into<String>) -> QueryPart {
        let node_var = node_var.into();
        let rel_node_var = format!("r_{node_var}");
        let mut query_part = QueryPart::default();

        if !self.is_empty() {
            query_part = query_part.match_clause(format!(
                "({node_var}) <-[:`{FROM_ENTITY}`]- ({rel_node_var})",
                FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE
            ));

            if let Some(mut relation_type) = self.relation_type {
                if let Some(space_id) = &self.space_id {
                    relation_type = relation_type.space_id(space_id.clone());
                }

                query_part.merge_mut(relation_type.into_query_part(
                    &rel_node_var,
                    system_ids::RELATION_TYPE_ATTRIBUTE,
                    self.space_version.clone(),
                ));
            }

            if let Some(mut to_id) = self.to_id {
                if let Some(space_id) = self.space_id {
                    to_id = to_id.space_id(space_id);
                }

                query_part.merge_mut(to_id.into_query_part(
                    &rel_node_var,
                    system_ids::RELATION_TO_ATTRIBUTE,
                    self.space_version,
                ));
            }
        }

        query_part
    }
}