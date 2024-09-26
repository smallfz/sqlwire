use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{Array, DataType, Expr, Map, MapEntry, Value as AstValue};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(BigDecimal),
    String(String),
    TypedString(String, String),
    Array(Vec<Value>),
    Dict(Vec<(Value, Value)>),
    Null,
}

impl From<Value> for Expr {
    fn from(v: Value) -> Self {
        match v {
            Value::Bool(bv) => Expr::Value(AstValue::Boolean(bv)),
            Value::Number(n) => Expr::Value(AstValue::Number(n, false)),
            Value::String(s) => Expr::Value(AstValue::SingleQuotedString(s)),
            Value::TypedString(typ, s) => Expr::TypedString {
                data_type: match typ.as_str() {
                    "datetime" | "date" => DataType::Date,
                    _ => DataType::Unspecified,
                },
                value: s,
            },
            Value::Array(mut array) => {
                let exprs = array.drain(..).map(|elem| elem.into());
                Expr::Array(Array {
                    elem: exprs.collect(),
                    named: false,
                })
            }
            Value::Dict(mut pairs) => {
                let entries = pairs
                    .drain(..)
                    .map(|(k, v)| MapEntry {
                        key: Box::new(k.into()),
                        value: Box::new(v.into()),
                    })
                    .collect();
                Expr::Map(Map { entries })
            }
            Value::Null => Expr::Value(AstValue::Null),
            // _ => Expr::Value(AstValue::Null),
        }
    }
}
