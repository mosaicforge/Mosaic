use neo4rs::BoltType;

use crate::mapping::Value;

use super::query_part::QueryPart;

pub fn value<T>(value: impl Into<T>) -> PropFilter<T> {
    PropFilter::default().value(value)
}

pub fn value_gt<T>(value: impl Into<T>) -> PropFilter<T> {
    PropFilter::default().value_gt(value)
}

pub fn value_gte<T>(value: impl Into<T>) -> PropFilter<T> {
    PropFilter::default().value_gte(value)
}

pub fn value_lt<T>(value: impl Into<T>) -> PropFilter<T> {
    PropFilter::default().value_lt(value)
}

pub fn value_lte<T>(value: impl Into<T>) -> PropFilter<T> {
    PropFilter::default().value_lte(value)
}

pub fn value_not<T>(value: impl Into<T>) -> PropFilter<T> {
    PropFilter::default().value_not(value)
}

pub fn value_in<T>(values: Vec<T>) -> PropFilter<T> {
    PropFilter::default().value_in(values)
}

pub fn value_not_in<T>(values: Vec<T>) -> PropFilter<T> {
    PropFilter::default().value_not_in(values)
}

impl From<&str> for PropFilter<String> {
    fn from(value: &str) -> Self {
        PropFilter::default().value(value.to_string())
    }
}

impl<T> From<T> for PropFilter<T> {
    fn from(value: T) -> Self {
        PropFilter::default().value(value)
    }
}

impl<T> From<Vec<T>> for PropFilter<T> {
    fn from(value: Vec<T>) -> Self {
        PropFilter::default().value_in(value)
    }
}

/// Filter for property P of node N
#[derive(Clone, Debug)]
pub struct PropFilter<T> {
    value: Option<T>,
    value_gt: Option<T>,
    value_gte: Option<T>,
    value_lt: Option<T>,
    value_lte: Option<T>,
    value_not: Option<T>,
    value_in: Option<Vec<T>>,
    value_not_in: Option<Vec<T>>,
    // or: Option<Vec<PropFilter<T>>>,
}

impl<T> Default for PropFilter<T> {
    fn default() -> Self {
        Self {
            value: None,
            value_gt: None,
            value_gte: None,
            value_lt: None,
            value_lte: None,
            value_not: None,
            value_in: None,
            value_not_in: None,
        }
    }
}

impl<T> PropFilter<T> {
    pub fn value(mut self, value: impl Into<T>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn value_mut(&mut self, value: impl Into<T>) {
        self.value = Some(value.into());
    }

    pub fn value_gt(mut self, value: impl Into<T>) -> Self {
        self.value_gt = Some(value.into());
        self
    }

    pub fn value_gt_mut(&mut self, value: impl Into<T>) {
        self.value_gt = Some(value.into());
    }

    pub fn value_gte(mut self, value: impl Into<T>) -> Self {
        self.value_gte = Some(value.into());
        self
    }

    pub fn value_gte_mut(&mut self, value: impl Into<T>) {
        self.value_gte = Some(value.into());
    }

    pub fn value_lt(mut self, value: impl Into<T>) -> Self {
        self.value_lt = Some(value.into());
        self
    }

    pub fn value_lt_mut(&mut self, value: impl Into<T>) {
        self.value_lt = Some(value.into());
    }

    pub fn value_lte(mut self, value: impl Into<T>) -> Self {
        self.value_lte = Some(value.into());
        self
    }

    pub fn value_lte_mut(&mut self, value: impl Into<T>) {
        self.value_lte = Some(value.into());
    }

    pub fn value_not(mut self, value: impl Into<T>) -> Self {
        self.value_not = Some(value.into());
        self
    }

    pub fn value_not_mut(&mut self, value: impl Into<T>) {
        self.value_not = Some(value.into());
    }

    pub fn value_in(mut self, values: Vec<T>) -> Self {
        self.value_in = Some(values);
        self
    }

    pub fn value_in_mut(&mut self, values: Vec<T>) {
        self.value_in = Some(values);
    }

    pub fn value_not_in(mut self, values: Vec<T>) -> Self {
        self.value_not_in = Some(values);
        self
    }

    pub fn value_not_in_mut(&mut self, values: Vec<T>) {
        self.value_not_in = Some(values);
    }
}

impl<T: Clone + Into<BoltType>> PropFilter<T> {
    /// Converts the filter into a query part.
    /// The `node_var` is the variable name of the node in the query.
    /// The `key` is the property key of the node.
    /// The `expr` is an optional expression to use instead of the property key.
    /// If `expr` is `None`, the node_var and key will be used as the expression to 
    /// filter, e.g. `{node_var}.{key} = $value`
    pub(crate) fn into_query_part(&self, node_var: &str, key: &str, expr: Option<&str>) -> QueryPart {
        let mut query_part = QueryPart::default();

        let expr = expr
            .map(|e| e.to_string())
            .unwrap_or(format!("{}.`{}`", node_var, key));

        if let Some(value) = &self.value {
            let param_key = format!("{node_var}_{key}_value");
            query_part = query_part
                .where_clause(format!("{expr} = ${param_key}"))
                .params(param_key, value.clone());
        }

        if let Some(value_gt) = &self.value_gt {
            let param_key = format!("{node_var}_{key}_value_gt");
            query_part = query_part
                .where_clause(format!("{expr} > ${param_key}"))
                .params(param_key, value_gt.clone());
        }

        if let Some(value_gte) = &self.value_gte {
            let param_key = format!("{node_var}_{key}_value_gte");
            query_part = query_part
                .where_clause(format!("{expr} >= ${param_key}"))
                .params(param_key, value_gte.clone());
        }

        if let Some(value_lt) = &self.value_lt {
            let param_key = format!("{node_var}_{key}_value_lt");
            query_part = query_part
                .where_clause(format!("{expr} < ${param_key}"))
                .params(param_key, value_lt.clone());
        }

        if let Some(value_lte) = &self.value_lte {
            let param_key = format!("{node_var}_{key}_value_lte");
            query_part = query_part
                .where_clause(format!("{expr} <= ${param_key}"))
                .params(param_key, value_lte.clone());
        }

        if let Some(value_not) = &self.value_not {
            let param_key = format!("{node_var}_{key}_value_not");
            query_part = query_part
                .where_clause(format!("{expr} <> ${param_key}"))
                .params(param_key, value_not.clone());
        }

        if let Some(value_in) = &self.value_in {
            let param_key = format!("{node_var}_{key}_value_in");
            query_part = query_part
                .where_clause(format!("{expr} IN ${param_key}"))
                .params(param_key, value_in.clone());
        }

        if let Some(value_not_in) = &self.value_not_in {
            let param_key = format!("{node_var}_{key}_value_not_in");
            query_part = query_part
                .where_clause(format!("{expr} NOT IN ${param_key}"))
                .params(param_key, value_not_in.clone());
        }

        query_part
    }
}

impl<T: Into<Value>> PropFilter<T> {
    pub fn as_string(self) -> PropFilter<String> {
        PropFilter {
            value: self.value.map(|v| v.into().value),
            value_gt: self.value_gt.map(|v| v.into().value),
            value_gte: self.value_gte.map(|v| v.into().value),
            value_lt: self.value_lt.map(|v| v.into().value),
            value_lte: self.value_lte.map(|v| v.into().value),
            value_not: self.value_not.map(|v| v.into().value),
            value_in: self
                .value_in
                .map(|v| v.into_iter().map(|v| v.into().value).collect()),
            value_not_in: self
                .value_not_in
                .map(|v| v.into_iter().map(|v| v.into().value).collect()),
        }
    }
}
