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
        }
    }
}

macro_rules! impl_from_int {
    ($t: ty) => {
        impl From<$t> for Value {
            fn from(i: $t) -> Self {
                Value::Number(BigDecimal::from(i))
            }
        }
    };
}

impl_from_int!(i8);
impl_from_int!(i16);
impl_from_int!(i32);
impl_from_int!(i64);
impl_from_int!(i128);
impl_from_int!(u8);
impl_from_int!(u16);
impl_from_int!(u32);
impl_from_int!(u64);
impl_from_int!(u128);

impl From<isize> for Value {
    fn from(i: isize) -> Self {
        Value::Number(BigDecimal::from(i64::try_from(i).unwrap_or(0i64)))
    }
}

impl From<usize> for Value {
    fn from(i: usize) -> Self {
        Value::Number(BigDecimal::from(u64::try_from(i).unwrap_or(0u64)))
    }
}

impl From<f32> for Value {
    fn from(i: f32) -> Self {
        Value::Number(BigDecimal::try_from(i).unwrap_or(BigDecimal::from(0)))
    }
}

impl From<f64> for Value {
    fn from(i: f64) -> Self {
        Value::Number(BigDecimal::try_from(i).unwrap_or(BigDecimal::from(0)))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<bool> for Value {
    fn from(bv: bool) -> Self {
        Value::Bool(bv)
    }
}
