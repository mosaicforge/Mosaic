use std::fmt::Display;

use juniper::GraphQLEnum;

#[derive(Debug, GraphQLEnum, PartialEq)]
pub enum ValueType {
    Text,
    Number,
    Checkbox,
    Url,
    Time,
    Point,
}

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueType::Text => write!(f, "TEXT"),
            ValueType::Number => write!(f, "NUMBER"),
            ValueType::Checkbox => write!(f, "CHECKBOX"),
            ValueType::Url => write!(f, "URL"),
            ValueType::Time => write!(f, "TIME"),
            ValueType::Point => write!(f, "POINT"),
        }
    }
}
