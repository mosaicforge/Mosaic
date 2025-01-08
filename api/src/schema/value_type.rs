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
