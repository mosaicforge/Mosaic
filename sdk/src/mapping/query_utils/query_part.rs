use std::collections::HashMap;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct QueryPart {
    /// Match clauses, e.g.: "(n {id: "123"})", "(n:Foo)", "(n)-[:REL]->(m)"
    pub(crate) match_clauses: Vec<String>,

    /// Where clauses, e.g.: "n.foo = $foo", "n.bar IN $bar"
    pub(crate) where_clauses: Vec<String>,

    /// With clauses, e.g.: "n", "n.bar", "m"
    pub(crate) return_clauses: Vec<String>,

    /// Order by clauses, e.g.: "n.foo", "n.bar DESC"
    pub(crate) order_by_clauses: Vec<String>,

    /// Parameters to be passed to the query
    pub(crate) params: HashMap<String, neo4rs::BoltType>,
}

impl QueryPart {
    // Builder methods
    pub fn match_clause(mut self, clause: &str) -> Self {
        self.match_clauses.push(clause.to_owned());
        self
    }

    pub fn where_clause(mut self, clause: &str) -> Self {
        self.where_clauses.push(clause.to_owned());
        self
    }

    pub fn return_clause(mut self, clause: &str) -> Self {
        self.return_clauses.push(clause.to_owned());
        self
    }

    pub fn order_by_clause(mut self, clause: &str) -> Self {
        self.order_by_clauses.push(clause.to_owned());
        self
    }

    pub fn params(mut self, key: String, value: neo4rs::BoltType) -> Self {
        self.params.insert(key, value);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.match_clauses.is_empty()
            && self.where_clauses.is_empty()
            && self.return_clauses.is_empty()
            && self.order_by_clauses.is_empty()
    }

    pub fn merge_mut(&mut self, other: QueryPart) {
        self.match_clauses.extend(other.match_clauses);
        self.where_clauses.extend(other.where_clauses);
        self.return_clauses.extend(other.return_clauses);
        self.order_by_clauses.extend(other.order_by_clauses);
        self.params.extend(other.params);
    }

    pub fn merge(mut self, other: QueryPart) -> Self {
        self.merge_mut(other);
        self
    }

    pub fn combine(parts: Vec<QueryPart>) -> QueryPart {
        parts
            .into_iter()
            .fold(QueryPart::default(), |acc, part| acc.merge(part))
    }

    pub fn query(&self) -> String {
        let mut query = String::new();

        self.match_clauses.iter().for_each(|clause| {
            query.push_str(&format!("MATCH {clause}\n"));
        });

        if !self.where_clauses.is_empty() {
            query.push_str("WHERE ");
            query.push_str(&self.where_clauses.join("\nAND "));
            query.push('\n');
        }

        if !self.return_clauses.is_empty() {
            query.push_str("RETURN ");
            query.push_str(&self.return_clauses.join(", "));
            query.push('\n');
        }

        if !self.order_by_clauses.is_empty() {
            query.push_str("ORDER BY ");
            query.push_str(&self.order_by_clauses.join(", "));
        }

        query
    }

    pub fn build(self) -> neo4rs::Query {
        let query_str = self.query();
        self.params
            .into_iter()
            .fold(neo4rs::query(&query_str), |query, (key, value)| {
                query.param(&key, value)
            })
    }
}

pub trait IntoQueryParts {
    fn into_query_parts(self) -> Vec<QueryPart>;

    fn merge(self) -> QueryPart
    where
        Self: Sized,
    {
        self.into_query_parts()
            .into_iter()
            .fold(QueryPart::default(), |acc, part| acc.merge(part))
    }

    fn build(self) -> neo4rs::Query
    where
        Self: Sized,
    {
        self.merge().build()
    }
}

pub trait IntoQueryPart {
    fn into_query_part(self) -> QueryPart;

    fn build(self) -> neo4rs::Query
    where
        Self: Sized,
    {
        self.into_query_part().build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_query_part() {
        let query_part = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: std::collections::HashMap::new(),
        };

        assert_eq!(
            query_part.query(),
            "MATCH (n)\nWHERE n.foo = $foo\nRETURN n\nORDER BY n.foo"
        );
    }

    #[test]
    fn test_query_part_params() {
        let query_part = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: HashMap::from([("foo".to_owned(), 123.into())]),
        };

        assert_eq!(query_part.params.len(), 1);
        assert_eq!(query_part.params.get("foo").unwrap(), &123.into());
    }

    #[test]
    fn test_query_part_merge() {
        let query_part1 = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: std::collections::HashMap::new(),
        };

        let query_part2 = super::QueryPart {
            match_clauses: vec!["(m)".to_owned()],
            where_clauses: vec!["m.bar = $bar".to_owned()],
            return_clauses: vec!["m".to_owned()],
            order_by_clauses: vec!["m.bar DESC".to_owned()],
            params: std::collections::HashMap::new(),
        };

        let merged = query_part1.merge(query_part2);

        assert_eq!(merged.query(), "MATCH (n)\nMATCH (m)\nWHERE n.foo = $foo\nAND m.bar = $bar\nRETURN n, m\nORDER BY n.foo, m.bar DESC");
    }

    #[test]
    fn test_query_part_merge_params() {
        let query_part1 = super::QueryPart {
            match_clauses: vec!["(n)".to_owned()],
            where_clauses: vec!["n.foo = $foo".to_owned()],
            return_clauses: vec!["n".to_owned()],
            order_by_clauses: vec!["n.foo".to_owned()],
            params: HashMap::from([("foo".to_owned(), 123.into())]),
        };

        let query_part2 = super::QueryPart {
            match_clauses: vec!["(m)".to_owned()],
            where_clauses: vec!["m.bar = $bar".to_owned()],
            return_clauses: vec!["m".to_owned()],
            order_by_clauses: vec!["m.bar DESC".to_owned()],
            params: HashMap::from([
                ("foo".to_owned(), 123.into()),
                ("bar".to_owned(), 456.into()),
            ]),
        };

        let merged = query_part1.merge(query_part2);

        assert_eq!(merged.query(), "MATCH (n)\nMATCH (m)\nWHERE n.foo = $foo\nAND m.bar = $bar\nRETURN n, m\nORDER BY n.foo, m.bar DESC");

        assert_eq!(merged.params.len(), 2);
        assert_eq!(merged.params.get("foo").unwrap(), &123.into());
        assert_eq!(merged.params.get("bar").unwrap(), &456.into());
    }
}
