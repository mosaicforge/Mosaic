use crate::system_ids;

#[derive(Clone, Debug, Default)]
pub struct RelationFilter {
    pub id: Option<String>,
    pub from_id: Option<String>,
    pub to_id: Option<String>,
    pub space_id: Option<String>,
    pub relation_type: Option<String>,
}

impl RelationFilter {
    pub fn query(&self) -> neo4rs::Query {
        let query = format!(
            r#"
            {match_clause}
            RETURN from, to, r, rt
            "#,
            match_clause = self.match_clause(),
        );

        neo4rs::query(&query)
            .param("id", self.id.clone().unwrap_or_default())
            .param("from_id", self.from_id.clone().unwrap_or_default())
            .param("to_id", self.to_id.clone().unwrap_or_default())
            .param("space_id", self.space_id.clone().unwrap_or_default())
            .param(
                "relation_type",
                self.relation_type.clone().unwrap_or_default(),
            )
    }

    fn match_clause(&self) -> String {
        format!(
            r#"
                MATCH ({match_clause_from}) <-[:`{FROM_ENTITY}`]- ({match_clause_relation}) -[:`{TO_ENTITY}`]-> ({match_clause_to})
                MATCH (r) -[:`{RELATION_TYPE}`]-> ({match_clause_relation_type})
            "#,
            FROM_ENTITY = system_ids::RELATION_FROM_ATTRIBUTE,
            TO_ENTITY = system_ids::RELATION_TO_ATTRIBUTE,
            RELATION_TYPE = system_ids::RELATION_TYPE_ATTRIBUTE,
            match_clause_from = self.match_clause_from(),
            match_clause_relation = self.match_clause_relation(),
            match_clause_to = self.match_clause_to(),
            match_clause_relation_type = self.match_clause_relation_type(),
        )
    }

    fn match_clause_from(&self) -> String {
        match (self.from_id.as_ref(), self.space_id.as_ref()) {
            (Some(_), Some(_)) => "(from {{id: $from_id, space_id: $space_id}})".to_string(),
            (None, Some(_)) => "(from {{space_id: $space_id}})".to_string(),
            (Some(_), None) => "(from {{id: $from_id}})".to_string(),
            (None, None) => "(from)".to_string(),
        }
    }

    fn match_clause_to(&self) -> String {
        match (self.to_id.as_ref(), self.space_id.as_ref()) {
            (Some(_), Some(_)) => "(to {{id: $to_id, space_id: $space_id}})".to_string(),
            (None, Some(_)) => "(to {{space_id: $space_id}})".to_string(),
            (Some(_), None) => "(to {{id: $to_id}})".to_string(),
            (None, None) => "(to)".to_string(),
        }
    }

    fn match_clause_relation(&self) -> String {
        // match (self.id.as_ref(), self.relation_type.as_ref()) {
        //     (Some(_), Some(rel_type)) => {
        //         format!("(r:`{rel_type}` {{id: $id}})", rel_type = rel_type)
        //     }
        //     (None, Some(rel_type)) => format!("(r:`{rel_type}`)", rel_type = rel_type),
        //     (Some(_), None) => "(r {id: $id})".to_string(),
        //     (None, None) => "(r)".to_string(),
        // }
        match self.id.as_ref() {
            Some(_) => "(r {id: $id})".to_string(),
            None => "(r)".to_string(),
        }
    }

    fn match_clause_relation_type(&self) -> String {
        match self.relation_type.as_ref() {
            Some(_) => "(rt {id: $relation_type})".to_string(),
            None => "(rt)".to_string(),
        }
    }
}
