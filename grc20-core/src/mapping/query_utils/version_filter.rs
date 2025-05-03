use super::query_builder::WhereClause;

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

    pub fn subquery(&self, var: &str) -> WhereClause {
        if let Some(version) = &self.version {
            let param_key = format!("{}_version", var);

            WhereClause::new(format!("{var}.min_version <= ${param_key} AND ({var}.max_version IS NULL OR {var}.max_version > ${param_key})"))
                .params(param_key, version.clone())
        } else {
            WhereClause::new(format!("{var}.max_version IS NULL"))
        }
    }
}
