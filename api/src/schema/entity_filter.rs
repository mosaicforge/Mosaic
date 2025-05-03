use juniper::GraphQLInputObject;

use grc20_core::{
    entity,
    mapping::{self, query_utils::PropFilter},
    relation, system_ids,
};

use crate::schema::EntityAttributeFilter;

use super::triple::ValueType;

/// Entity filter input object
///
/// ```graphql
/// query {
///     entities(where: {
///         space_id: "BJqiLPcSgfF8FRxkFr76Uy",
///         types_contain: ["XG26vy98XAA6cR6DosTALk", "XG26vy98XAA6cR6DosTALk"],
///         attributes_contain: [
///             {id: "XG26vy98XAA6cR6DosTALk", value: "value", value_type: TEXT},
///         ]
///     })
/// }
/// ```
#[derive(Debug, GraphQLInputObject)]
pub struct EntityFilter {
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    /// Exact match for the entity types
    // pub types: Option<Vec<String>>,
    // pub types_not: Option<Vec<String>>,
    pub types_contains: Option<Vec<String>>,
    pub types_not_contains: Option<Vec<String>>,

    pub attributes: Option<Vec<EntityAttributeFilter>>,
}

impl EntityFilter {
    fn id_filter(&self) -> PropFilter<String> {
        let mut filter = PropFilter::default();

        if let Some(id) = &self.id {
            filter = filter.value(id);
        }

        if let Some(id_not) = &self.id_not {
            filter = filter.value_not(id_not);
        }

        if let Some(id_in) = &self.id_in {
            filter = filter.value_in(id_in.clone());
        }

        if let Some(id_not_in) = &self.id_not_in {
            filter = filter.value_not_in(id_not_in.clone());
        }

        filter
    }

    fn types_filter(&self) -> mapping::EntityRelationFilter {
        let mut filter = mapping::EntityRelationFilter::default();

        // if let Some(types) = &self.types {
        //     filter = filter.to_id(EdgeFilter::default().to_id(prop_filter::value_in(types.clone())));
        // }

        // if let Some(types_not) = &self.types_not {
        //     filter = filter.to_id(EdgeFilter::default().to_id_not_in(types_not.clone()));
        // }

        if self.types_contains.is_some() || self.types_not_contains.is_some() {
            filter = filter.relation_type(system_ids::TYPES_ATTRIBUTE);
        }

        if let Some(types_contains) = &self.types_contains {
            filter = filter.to_id(types_contains.clone());
        }

        if let Some(types_not_contains) = &self.types_not_contains {
            filter = filter.to_id(types_not_contains.clone());
        }

        filter
    }
}

impl From<EntityFilter> for mapping::EntityFilter {
    fn from(filter: EntityFilter) -> Self {
        // TODO: Add types filter
        mapping::EntityFilter::default()
            .id(filter.id_filter())
            .relations(filter.types_filter())
            .attributes(
                filter
                    .attributes
                    .unwrap_or_default()
                    .into_iter()
                    .map(|attribute| attribute.into()),
            )
    }
}

#[derive(Debug, GraphQLInputObject)]
pub struct AttributeFilter {
    pub(crate) value_type: Option<ValueType>,
}

/// Filters the outgoing relations of the entity
#[derive(Debug, GraphQLInputObject)]
pub struct EntityRelationFilter {
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    pub to_id: Option<String>,
    pub to_id_not: Option<String>,
    pub to_id_in: Option<Vec<String>>,
    pub to_id_not_in: Option<Vec<String>>,

    pub relation_type: Option<String>,
    pub relation_type_not: Option<String>,
    pub relation_type_in: Option<Vec<String>>,
    pub relation_type_not_in: Option<Vec<String>>,

    /// Filter the relations by the entity they point to
    pub to_: Option<EntityFilter>,
    // pub attributes: Option<Vec<EntityAttributeFilter>>,
}

impl EntityRelationFilter {
    fn id_filter(&self) -> PropFilter<String> {
        let mut filter = PropFilter::default();

        if let Some(id) = &self.id {
            filter = filter.value(id);
        }

        if let Some(id_not) = &self.id_not {
            filter = filter.value_not(id_not);
        }

        if let Some(id_in) = &self.id_in {
            filter = filter.value_in(id_in.clone());
        }

        if let Some(id_not_in) = &self.id_not_in {
            filter = filter.value_not_in(id_not_in.clone());
        }

        filter
    }

    fn to_id_filter(&self) -> PropFilter<String> {
        let mut filter = PropFilter::default();

        if let Some(to_id) = &self.to_id {
            filter = filter.value(to_id);
        }

        if let Some(to_id_not) = &self.to_id_not {
            filter = filter.value_not(to_id_not);
        }

        if let Some(to_id_in) = &self.to_id_in {
            filter = filter.value_in(to_id_in.clone());
        }

        if let Some(to_id_not_in) = &self.to_id_not_in {
            filter = filter.value_not_in(to_id_not_in.clone());
        }

        filter
    }

    fn relation_type_filter(&self) -> PropFilter<String> {
        let mut filter = PropFilter::default();

        if let Some(relation_type) = &self.relation_type {
            filter = filter.value(relation_type);
        }

        if let Some(relation_type_not) = &self.relation_type_not {
            filter = filter.value_not(relation_type_not);
        }

        if let Some(relation_type_in) = &self.relation_type_in {
            filter = filter.value_in(relation_type_in.clone());
        }

        if let Some(relation_type_not_in) = &self.relation_type_not_in {
            filter = filter.value_not_in(relation_type_not_in.clone());
        }

        filter
    }

    pub fn apply_filter<T>(&self, query: relation::FindManyQuery<T>) -> relation::FindManyQuery<T> {
        query.filter(
            relation::RelationFilter::default()
                .id(self.id_filter())
                .to_(entity::EntityFilter::default().id(self.to_id_filter()))
                .relation_type(entity::EntityFilter::default().id(self.relation_type_filter())),
        )
    }
}
