use super::query_part::QueryPart;

#[derive(Debug, Default, Clone)]
pub struct VersionFilter {
    version: Option<String>,
}

impl VersionFilter {
    pub fn new(version: Option<String>) -> Self {
        Self { version }
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn version_mut(&mut self, version: String) {
        self.version = Some(version);
    }

    pub fn version_opt(&mut self, version: Option<String>) {
        self.version = version;
    }

    pub fn compile(&self, var: &str) -> QueryPart {
        let query_part = QueryPart::default();

        let param_key = format!("{}_version", var);

        if let Some(version) = &self.version {
            query_part.where_clause(format!("{var}.min_version <= ${param_key} AND ({var}.max_version IS NULL OR {var}.max_version > ${param_key})"))
                .params(param_key, version.clone())
        } else {
            query_part.where_clause(format!("{var}.max_version IS NULL"))
        }
    }
}
