use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeedValue {
    pub value: Value,
    pub range: Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Block(Vec<Entry>),
    TaggedBlock(String, Vec<Entry>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operator {
    Equals,      // =
    LessThan,    // <
    GreaterThan, // >
    NotEquals,   // !=
    LessOrEqual, // <=
    GreaterOrEqual, // >=
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub key: String,
    pub key_range: Range,
    pub operator: Operator,
    pub operator_range: Range,
    pub value: NodeedValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Entry {
    Assignment(Assignment),
    Value(NodeedValue),
    Comment(String, Range),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    pub entries: Vec<Entry>,
}