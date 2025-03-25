use crate::{mapping::entity_node::EntityRelationFilter, system_ids};

use super::{prop_filter, EdgeFilter};

#[derive(Clone, Debug, Default)]
pub struct TypesFilter {
    types_contains: Vec<String>,
}

impl TypesFilter {
    pub fn r#type(mut self, r#type: impl Into<String>) -> Self {
        self.types_contains.push(r#type.into());
        self
    }

    pub fn types(mut self, mut types: Vec<String>) -> Self {
        self.types_contains.append(&mut types);
        self
    }
}

impl From<TypesFilter> for EntityRelationFilter {
    fn from(types_filter: TypesFilter) -> Self {
        let mut filter = EntityRelationFilter::default();

        if !types_filter.types_contains.is_empty() {
            filter = filter.relation_type(
                EdgeFilter::default().to_id(prop_filter::value(system_ids::TYPES_ATTRIBUTE)),
            );

            filter = filter.to_id(
                EdgeFilter::default().to_id(prop_filter::value_in(types_filter.types_contains)),
            );
        }

        filter
    }
}
