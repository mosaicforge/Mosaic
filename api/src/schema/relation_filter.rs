use juniper::GraphQLInputObject;

use grc20_core::{entity, mapping::{self, relation, EntityNode, RelationEdge}};

use crate::schema::{EntityAttributeFilter, EntityFilter};

/// Relation filter input object
#[derive(Debug, GraphQLInputObject)]
pub struct RelationFilter {
    /// Filter the relations by their id
    pub id: Option<String>,
    pub id_not: Option<String>,
    pub id_in: Option<Vec<String>>,
    pub id_not_in: Option<Vec<String>>,

    /// Filter the relations by their relation type
    pub relation_type: Option<String>,
    pub relation_type_not: Option<String>,
    pub relation_type_in: Option<Vec<String>>,
    pub relation_type_not_in: Option<Vec<String>>,

    /// Filter the relations by the entity they point to
    pub to_: Option<EntityFilter>,

    /// Filter the relations by the entity they point from
    pub from_: Option<EntityFilter>,

    /// Filter the relations by their attributes
    pub attributes: Option<Vec<EntityAttributeFilter>>,
}

impl RelationFilter {
    pub fn id_filter(&self) -> mapping::PropFilter<String> {
        let mut filter = mapping::PropFilter::default();

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

    pub fn relation_type_filter(&self) -> mapping::PropFilter<String> {
        let mut filter = mapping::PropFilter::default();

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

    pub fn apply_filter(
        self,
        mut query: relation::FindManyQuery<RelationEdge<EntityNode>>,
    ) -> relation::FindManyQuery<RelationEdge<EntityNode>> {
        query = query
            .filter(
                relation::RelationFilter::default()
                    .id(self.id_filter())
                    .relation_type(entity::EntityFilter::default().id(self.relation_type_filter())),
            );

        // if let Some(attributes) = self.attributes {
        //     for attr in attributes {
        //         query = attr.add_to_relation_query(query);
        //     }
        // }

        // if let Some(to_) = self.to_ {
        //     query = query.to(|to_filter| to_.add_to_entity_query(to_filter));
        // }

        // if let Some(from_) = self.from_ {
        //     query = query.from(|from_filter| from_.add_to_entity_query(from_filter));
        // }

        query
    }
}
