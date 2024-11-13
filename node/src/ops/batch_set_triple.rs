use crate::ops::Value;

pub struct BatchSetTriples {
    pub entity_id: String,
    pub type_id: String,
    pub values: Vec<Value>,
}
