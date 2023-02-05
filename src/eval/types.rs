use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Value {
    AttrSet(HashMap<String, Value>),
    Bool(bool),
    Float(f64),
    Int(i64),
    List(Vec<Value>),
    Str(String),
}

pub type EvalResult = Result<Value, String>;
