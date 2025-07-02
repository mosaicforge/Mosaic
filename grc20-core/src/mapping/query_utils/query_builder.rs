use std::collections::HashMap;

pub trait Subquery {
    fn statements(&self) -> Vec<String>;

    fn params(&self) -> HashMap<String, neo4rs::BoltType>;

    fn compile(&self) -> String {
        self.statements().join("\n")
    }

    fn build(&self) -> neo4rs::Query {
        let mut query = neo4rs::query(&self.compile());

        for (key, value) in self.params() {
            query = query.param(&key, value);
        }

        query
    }
}

impl Subquery for String {
    fn statements(&self) -> Vec<String> {
        vec![self.clone()]
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        HashMap::new()
    }
}

impl Subquery for &str {
    fn statements(&self) -> Vec<String> {
        vec![self.to_string()]
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        HashMap::new()
    }
}

impl Subquery for Vec<String> {
    fn statements(&self) -> Vec<String> {
        self.clone()
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        HashMap::new()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QueryBuilder {
    pub statements: Vec<String>,
    pub params: HashMap<String, neo4rs::BoltType>,
}

impl QueryBuilder {
    pub fn subquery(mut self, subquery: impl Subquery) -> Self {
        self.statements.extend(subquery.statements());
        self.params.extend(subquery.params().clone());
        self
    }

    pub fn subquery_opt(mut self, subquery: Option<impl Subquery>) -> Self {
        if let Some(subquery) = subquery {
            self.statements.extend(subquery.statements());
            self.params.extend(subquery.params().clone());
        }
        self
    }

    pub fn subqueries(mut self, subqueries: Vec<impl Subquery>) -> Self {
        for subquery in subqueries {
            self.statements.extend(subquery.statements());
            self.params.extend(subquery.params().clone());
        }
        self
    }

    pub fn params(mut self, key: impl Into<String>, value: impl Into<neo4rs::BoltType>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    pub fn with(mut self, vars: Vec<String>, subquery: impl Subquery) -> Self {
        self.statements.push(format!("WITH {}", vars.join(", ")));
        self.statements.extend(subquery.statements());
        self.params.extend(subquery.params().clone());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.statements.push(format!("LIMIT {limit}"));
        self
    }

    pub fn skip(mut self, skip: usize) -> Self {
        self.statements.push(format!("SKIP {skip}"));
        self
    }

    pub fn skip_opt(mut self, skip: Option<usize>) -> Self {
        if let Some(skip) = skip {
            self.statements.push(format!("SKIP {skip}"));
        }
        self
    }

    pub fn r#return(mut self, return_clause: impl Into<String>) -> impl Subquery {
        self.statements
            .push(format!("RETURN {}", return_clause.into()));
        self
    }
}

impl Subquery for QueryBuilder {
    fn statements(&self) -> Vec<String> {
        self.statements.clone()
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        self.params.clone()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MatchQuery {
    pub(crate) match_clause: String,
    pub(crate) optional: bool,
    pub(crate) where_clauses: Vec<String>,
    pub(crate) rename: Option<NamePair>,
    pub(crate) params: HashMap<String, neo4rs::BoltType>,
}

impl MatchQuery {
    pub fn new(match_clause: impl Into<String>) -> Self {
        Self {
            match_clause: match_clause.into(),
            optional: false,
            rename: None,
            where_clauses: Vec::new(),
            params: HashMap::new(),
        }
    }

    pub fn new_optional(match_clause: impl Into<String>) -> Self {
        Self {
            match_clause: match_clause.into(),
            optional: true,
            rename: None,
            where_clauses: Vec::new(),
            params: HashMap::new(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn r#rename(mut self, rename: impl Into<Rename>) -> Self {
        let rename_clause: Rename = rename.into();
        self.rename = Some(rename_clause.name_pair);
        self.params.extend(rename_clause.params);
        self
    }

    pub fn r#where(mut self, clause: impl Into<WhereClause>) -> Self {
        let where_clause: WhereClause = clause.into();
        self.where_clauses.extend(where_clause.clauses);
        self.params.extend(where_clause.params);
        self
    }

    pub fn where_opt(mut self, clause: Option<impl Into<WhereClause>>) -> Self {
        if let Some(clause) = clause {
            let where_clause: WhereClause = clause.into();
            self.where_clauses.extend(where_clause.clauses);
            self.params.extend(where_clause.params);
        }
        self
    }

    pub fn params(mut self, key: impl Into<String>, value: impl Into<neo4rs::BoltType>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

impl Subquery for MatchQuery {
    fn statements(&self) -> Vec<String> {
        let mut statements = Vec::new();

        if let Some(rename) = &self.rename {
            statements.push(format!("WITH {} AS {}", rename.from_name, rename.to_name))
        };

        statements.push(if self.optional {
            format!("OPTIONAL MATCH {}", self.match_clause)
        } else {
            format!("MATCH {}", self.match_clause)
        });

        match &self.where_clauses.as_slice() {
            [] => (),
            [clause, rest @ ..] => {
                statements.push(format!("WHERE {clause}"));
                for rest_clause in rest {
                    statements.push(format!("AND {rest_clause}"));
                }
            }
        }

        statements
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        self.params.clone()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NamePair {
    from_name: String,
    to_name: String,
}

impl NamePair {
    pub fn new(from_name: impl Into<String>, to_name: impl Into<String>) -> Self {
        Self {
            from_name: from_name.into(),
            to_name: to_name.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rename {
    name_pair: NamePair,
    params: HashMap<String, neo4rs::BoltType>,
}

impl Rename {
    pub fn new(name_pair: impl Into<NamePair>) -> Self {
        Self {
            name_pair: name_pair.into(),
            params: HashMap::new(),
        }
    }

    pub fn set_param(mut self, key: impl Into<String>, value: impl Into<neo4rs::BoltType>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

impl Subquery for Rename {
    fn statements(&self) -> Vec<String> {
        vec![format!(
            "{} AS {}",
            self.name_pair.from_name, self.name_pair.to_name
        )]
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        self.params.clone()
    }
}
impl Rename {
    pub fn name_pair(mut self, name_pair: impl Into<NamePair>) -> Self {
        self.name_pair = name_pair.into();
        self
    }

    pub fn name_pair_opt(mut self, name_pair: Option<impl Into<NamePair>>) -> Self {
        if let Some(name_pair) = name_pair {
            self.name_pair = name_pair.into();
        }
        self
    }
}

impl From<NamePair> for Rename {
    fn from(rename: NamePair) -> Self {
        Self {
            name_pair: rename,
            params: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WhereClause {
    pub clauses: Vec<String>,
    pub params: HashMap<String, neo4rs::BoltType>,
}

impl WhereClause {
    pub fn new(clause: impl Into<String>) -> Self {
        Self {
            clauses: vec![clause.into()],
            params: HashMap::new(),
        }
    }

    pub fn clause(mut self, clause: impl Into<String>) -> Self {
        self.clauses.push(clause.into());
        self
    }

    pub fn clauses(mut self, clauses: Vec<String>) -> Self {
        self.clauses.extend(clauses);
        self
    }

    pub fn clause_opt(mut self, clause: Option<impl Into<String>>) -> Self {
        if let Some(clause) = clause {
            self.clauses.push(clause.into());
        }
        self
    }

    pub fn set_param(mut self, key: impl Into<String>, value: impl Into<neo4rs::BoltType>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

impl Subquery for WhereClause {
    fn statements(&self) -> Vec<String> {
        match &self.clauses.as_slice() {
            [] => vec![],
            [clause, rest @ ..] => {
                let mut statements = vec![format!("WHERE {clause}")];
                for rest_clause in rest {
                    statements.push(format!("AND {rest_clause}"));
                }
                statements
            }
        }
    }

    fn params(&self) -> HashMap<String, neo4rs::BoltType> {
        self.params.clone()
    }
}

impl From<String> for WhereClause {
    fn from(clause: String) -> Self {
        Self {
            clauses: vec![clause],
            params: HashMap::new(),
        }
    }
}

impl From<&str> for WhereClause {
    fn from(clause: &str) -> Self {
        Self {
            clauses: vec![clause.to_string()],
            params: HashMap::new(),
        }
    }
}

impl From<Vec<String>> for WhereClause {
    fn from(clauses: Vec<String>) -> Self {
        Self {
            clauses,
            params: HashMap::new(),
        }
    }
}
