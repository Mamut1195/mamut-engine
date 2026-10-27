//! Parsing without duplicate keys and numerical normalization at the JSON boundary.
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};
use std::fmt;

use crate::ApiError;

struct Strict(Value);

impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct JsonVisitor;
        impl<'de> Visitor<'de> for JsonVisitor {
            type Value = Strict;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("Finite JSON without duplicate keys")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Strict, E> {
                Ok(Strict(Value::Bool(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<Strict, E> {
                Ok(Strict(Value::Number(v.into())))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<Strict, E> {
                Ok(Strict(Value::Number(v.into())))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<Strict, E> {
                // Floating-point zero retains its type; only its sign is removed.
                let v = if v == 0.0 { 0.0 } else { v };
                Number::from_f64(v)
                    .map(|n| Strict(Value::Number(n)))
                    .ok_or_else(|| E::custom("Non-finite number"))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Strict, E> {
                Ok(Strict(Value::String(v.into())))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Strict, E> {
                Ok(Strict(Value::Null))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Strict, A::Error> {
                let mut values = Vec::new();
                while let Some(Strict(v)) = seq.next_element()? {
                    values.push(v);
                }
                Ok(Strict(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Strict, A::Error> {
                let mut values = Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom(format!("Duplicate key: {key}")));
                    }
                    let Strict(v) = map.next_value()?;
                    values.insert(key, v);
                }
                Ok(Strict(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(JsonVisitor)
    }
}

pub(crate) fn parse(input: &str) -> Result<Value, ApiError> {
    crate::limits::preflight(input)?;
    serde_json::from_str::<Strict>(input)
        .map(|v| v.0)
        .map_err(|e| ApiError::InvalidInput(e.to_string()))
}

pub(crate) fn canonical(input: &str) -> Result<String, ApiError> {
    canonical_value(parse(input)?)
}

pub(crate) fn canonical_value(mut value: Value) -> Result<String, ApiError> {
    // Explicit ordering even if another interface enables serde_json preserve_order.
    fn sort(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for v in map.values_mut() {
                    sort(v);
                }
                map.sort_keys();
            }
            Value::Array(values) => values.iter_mut().for_each(sort),
            Value::Number(n) if n.is_f64() && n.as_f64() == Some(0.0) => {
                *value = serde_json::json!(0.0);
            }
            _ => {}
        }
    }
    sort(&mut value);
    serde_json::to_string(&value).map_err(|e| ApiError::InvalidInput(e.to_string()))
}
