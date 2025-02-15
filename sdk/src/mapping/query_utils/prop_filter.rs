use neo4rs::BoltType;

use super::query_part::QueryPart;

/// Filter for property P of node N
#[derive(Debug, Clone)]
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

impl<T> PropFilter<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            value_gt: None,
            value_gte: None,
            value_lt: None,
            value_lte: None,
            value_not: None,
            value_in: None,
            value_not_in: None,
            // or: None,
        }
    }

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
    pub(crate) fn into_query_part(self, node_var: &str, key: &str) -> QueryPart {
        let mut query_part = QueryPart::default();

        if let Some(value) = self.value {
            let param_key = format!("{node_var}_{key}_value");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` = ${param_key}"))
                .params(param_key, value);
        }

        if let Some(value_gt) = self.value_gt {
            let param_key = format!("{node_var}_{key}_value_gt");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` > ${param_key}"))
                .params(param_key, value_gt);
        }

        if let Some(value_gte) = self.value_gte {
            let param_key = format!("{node_var}_{key}_value_gte");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` >= ${param_key}"))
                .params(param_key, value_gte);
        }

        if let Some(value_lt) = self.value_lt {
            let param_key = format!("{node_var}_{key}_value_lt");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` < ${param_key}"))
                .params(param_key, value_lt);
        }

        if let Some(value_lte) = self.value_lte {
            let param_key = format!("{node_var}_{key}_value_lte");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` <= ${param_key}"))
                .params(param_key, value_lte);
        }

        if let Some(value_not) = self.value_not {
            let param_key = format!("{node_var}_{key}_value_not");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` <> ${param_key}"))
                .params(param_key, value_not);
        }

        if let Some(value_in) = self.value_in {
            let param_key = format!("{node_var}_{key}_value_in");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` IN ${param_key}"))
                .params(param_key, value_in);
        }

        if let Some(value_not_in) = self.value_not_in {
            let param_key = format!("{node_var}_{key}_value_not_in");
            query_part = query_part
                .where_clause(&format!("{node_var}.`{key}` NOT IN ${param_key}"))
                .params(param_key, value_not_in);
        }

        query_part
    }
}
