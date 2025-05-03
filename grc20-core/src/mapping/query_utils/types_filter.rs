use crate::{mapping::EntityRelationFilter, system_ids};

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
            filter = filter.relation_type(system_ids::TYPES_ATTRIBUTE);

            if let [r#type] = &types_filter.types_contains[..] {
                filter = filter.to_id(r#type.to_string());
            } else {
                filter = filter.to_id(types_filter.types_contains);
            }
        }

        filter
    }
}
