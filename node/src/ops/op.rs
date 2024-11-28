use std::fmt::Display;

use crate::kg::client::Client;
use futures::future::BoxFuture;
use kg_core::pb::grc20;

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Text(String),
    Number(String),
    Entity(String),
    Uri(String),
    Checkbox(bool),
    Time(String),        // TODO: Change to proper type
    GeoLocation(String), // TODO: Change to proper type
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Text(value) => write!(f, "{}", value),
            Value::Number(value) => write!(f, "{}", value),
            Value::Entity(value) => write!(f, "{}", value),
            Value::Uri(value) => write!(f, "{}", value),
            Value::Checkbox(value) => write!(f, "{}", value),
            Value::Time(value) => write!(f, "{}", value),
            Value::GeoLocation(value) => write!(f, "{}", value),
        }
    }
}

impl From<grc20::Value> for Value {
    fn from(value: grc20::Value) -> Self {
        match value.r#type() {
            grc20::ValueType::Text => Value::Text(value.value),
            grc20::ValueType::Number => Value::Number(value.value),
            grc20::ValueType::Entity => Value::Entity(value.value),
            grc20::ValueType::Uri => Value::Uri(value.value),
            grc20::ValueType::Checkbox => Value::Checkbox(value.value.parse().unwrap_or(false)),
            grc20::ValueType::Time => Value::Time(value.value),
            grc20::ValueType::GeoLocation => Value::GeoLocation(value.value),
            _ => Value::Null,
        }
    }
}

impl From<&grc20::Value> for Value {
    fn from(value: &grc20::Value) -> Self {
        match value.r#type() {
            grc20::ValueType::Text => Value::Text(value.value.clone()),
            grc20::ValueType::Number => Value::Number(value.value.clone()),
            grc20::ValueType::Entity => Value::Entity(value.value.clone()),
            grc20::ValueType::Uri => Value::Uri(value.value.clone()),
            grc20::ValueType::Checkbox => Value::Checkbox(value.value.parse().unwrap_or(false)),
            grc20::ValueType::Time => Value::Time(value.value.clone()),
            grc20::ValueType::GeoLocation => Value::GeoLocation(value.value.clone()),
            _ => Value::Null,
        }
    }
}

impl From<Value> for neo4rs::BoltType {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => neo4rs::BoltType::Null(neo4rs::BoltNull),
            Value::Text(value) => neo4rs::BoltType::String(value.into()),
            Value::Number(value) => neo4rs::BoltType::String(value.into()),
            Value::Entity(value) => neo4rs::BoltType::String(value.into()),
            Value::Uri(value) => neo4rs::BoltType::String(value.into()),
            Value::Checkbox(value) => neo4rs::BoltType::Boolean(neo4rs::BoltBoolean::new(value)),
            Value::Time(value) => neo4rs::BoltType::String(value.into()),
            Value::GeoLocation(value) => neo4rs::BoltType::String(value.into()),
        }
    }
}

pub struct Op(Box<dyn KgOpDyn>);

impl Op {
    pub fn new<T: KgOp + 'static>(op: T) -> Self {
        Op(Box::new(op))
    }

    pub fn null() -> Self {
        Op(Box::new(NullOp))
    }

    pub fn apply_op<'a>(
        &'a self,
        kg: &'a Client,
        space_id: &'a str,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        self.0.apply_op(kg, space_id)
    }
}

pub trait KgOp: Send {
    fn apply_op(
        &self,
        kg: &Client,
        space_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

pub trait KgOpDyn: Send {
    fn apply_op<'a>(
        &'a self,
        kg: &'a Client,
        space_id: &'a str,
    ) -> BoxFuture<'a, anyhow::Result<()>>;
}

impl<T: KgOp> KgOpDyn for T {
    fn apply_op<'a>(
        &'a self,
        kg: &'a Client,
        space_id: &'a str,
    ) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(self.apply_op(kg, space_id))
    }
}

pub struct NullOp;

impl KgOp for NullOp {
    async fn apply_op(&self, _kg: &Client, _space_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}
