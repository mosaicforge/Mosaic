use super::query_part::QueryPart;

#[derive(Debug, Default, Clone)]
pub struct VersionFilter {
    version: Option<i64>,
}

impl VersionFilter {
    pub fn new(version: Option<i64>) -> Self {
        Self { version }
    }

    pub fn version(mut self, version: i64) -> Self {
        self.version = Some(version);
        self
    }

    pub fn version_mut(&mut self, version: i64) {
        self.version = Some(version);
    }

    pub fn into_query_part(self, var: &str) -> QueryPart {
        let query_part = QueryPart::default();

        let param_key = format!("{}_version", var);

        if let Some(version) = self.version {
            query_part.where_clause(format!("{var}.min_version <= ${param_key} AND ({var}.max_version IS NULL OR {var}.max_version > ${param_key})"))
                .params(param_key, version)
        } else {
            query_part.where_clause(format!("{var}.max_version IS NULL"))
        }
    }
}
