use std::ffi::CStr;

use chrono::DateTime;
use surrealdb_core::sql;
use surrealdb_core::sql::Value as sdbValue;

pub use crate::{array::Array, number::Number, object::Object};
use crate::{bytes::Bytes, string::string_t, thing::Thing, utils::CStringExt2, uuid::Uuid};

use super::duration::Duration;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub enum Value {
    #[default]
    SR_VALUE_NONE,
    SR_VALUE_NULL,
    SR_VALUE_BOOL(bool),
    SR_VALUE_NUMBER(Number),
    SR_VALUE_STRAND(string_t),
    SR_VALUE_DURATION(Duration),
    SR_VALUE_DATETIME(string_t),
    SR_VALUE_UUID(Uuid),
    SR_VALUE_ARRAY(Box<Array>),
    SR_VALUE_OBJECT(Object),
    // Geometry(Geometry),
    SR_VALUE_BYTES(Bytes),
    SR_VALUE_THING(Thing),
}

impl From<sdbValue> for Value {
    fn from(value: sdbValue) -> Self {
        match value {
            sdbValue::None => Value::SR_VALUE_NONE,
            sdbValue::Null => Value::SR_VALUE_NULL,
            sdbValue::Bool(b) => Value::SR_VALUE_BOOL(b),
            sdbValue::Number(n) => match n {
                sql::Number::Int(i) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_INT(i)),
                sql::Number::Float(f) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(f)),
                sql::Number::Decimal(_) => todo!(),
                _ => todo!(),
            },
            sdbValue::Strand(s) => Value::SR_VALUE_STRAND(s.0.to_string_t()),
            sdbValue::Duration(d) => Value::SR_VALUE_DURATION(d.into()),
            sdbValue::Datetime(dt) => Value::SR_VALUE_DATETIME(dt.to_rfc3339().to_string_t()),
            sdbValue::Uuid(u) => Value::SR_VALUE_UUID(u.into()),
            // unecssary box see: https://github.com/mozilla/cbindgen/issues/981
            sdbValue::Array(a) => Value::SR_VALUE_ARRAY(Box::new(a.into())),
            sdbValue::Object(o) => Value::SR_VALUE_OBJECT(o.into()),
            sdbValue::Geometry(_) => todo!(),
            sdbValue::Bytes(b) => Value::SR_VALUE_BYTES(b.into()),
            sdbValue::Thing(t) => Value::SR_VALUE_THING(t.into()),
            _ => unimplemented!("other variants shouldn't be returned"),
        }
    }
}

impl From<&sdbValue> for Value {
    fn from(value: &sdbValue) -> Self {
        match value {
            sdbValue::None => Value::SR_VALUE_NONE,
            sdbValue::Null => Value::SR_VALUE_NULL,
            sdbValue::Bool(b) => Value::SR_VALUE_BOOL(*b),
            sdbValue::Number(n) => match n {
                sql::Number::Int(i) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_INT(*i)),
                sql::Number::Float(f) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(*f)),
                sql::Number::Decimal(_) => todo!(),
                _ => todo!(),
            },
            sdbValue::Strand(s) => Value::SR_VALUE_STRAND(s.0.as_str().to_string_t()),
            sdbValue::Duration(d) => Value::SR_VALUE_DURATION(d.clone().into()),
            sdbValue::Datetime(dt) => Value::SR_VALUE_DATETIME(dt.to_rfc3339().to_string_t()),
            sdbValue::Uuid(u) => Value::SR_VALUE_UUID(u.clone().into()),
            // unecssary box see: https://github.com/mozilla/cbindgen/issues/981
            sdbValue::Array(a) => Value::SR_VALUE_ARRAY(Box::new(a.into())),
            sdbValue::Object(o) => Value::SR_VALUE_OBJECT(o.into()),
            sdbValue::Geometry(_) => todo!(),
            sdbValue::Bytes(b) => Value::SR_VALUE_BYTES(b.clone().into()),
            sdbValue::Thing(t) => Value::SR_VALUE_THING(t.into()),
            _ => unimplemented!("other variants shouldn't be returned"),
        }
    }
}
impl From<surrealdb::Value> for Value {
    fn from(value: surrealdb::Value) -> Self {
        let value: sdbValue = value.into_inner();
        Self::from(value)
    }
}
impl From<&surrealdb::Value> for Value {
    fn from(value: &surrealdb::Value) -> Self {
        // TODO: change to into_inner_ref when merged
        // SAFETY: surrealdb::Value is a transparent srapper
        let value: &sdbValue = unsafe { std::mem::transmute(value) };
        Self::from(value)
    }
}

impl From<Value> for sdbValue {
    fn from(value: Value) -> Self {
        match value {
            Value::SR_VALUE_NONE => sdbValue::None,
            Value::SR_VALUE_NULL => sdbValue::Null,
            Value::SR_VALUE_BOOL(b) => sdbValue::Bool(b),
            Value::SR_VALUE_NUMBER(n) => sdbValue::Number(n.into()),
            Value::SR_VALUE_STRAND(s) => sdbValue::Strand(String::from(s).into()),
            Value::SR_VALUE_DURATION(d) => sdbValue::Duration(d.into()),
            Value::SR_VALUE_DATETIME(d) => {
                let cstr = unsafe { CStr::from_ptr(d.0) };
                sdbValue::Datetime(
                    DateTime::parse_from_rfc3339(cstr.to_string_lossy().as_ref())
                        .unwrap_or_default()
                        .to_utc()
                        .into(),
                )
            }
            Value::SR_VALUE_UUID(u) => sdbValue::Uuid(u.into()),
            Value::SR_VALUE_ARRAY(a) => sdbValue::Array((*a).into()),
            Value::SR_VALUE_OBJECT(o) => sdbValue::Object(o.into()),
            Value::SR_VALUE_BYTES(b) => sdbValue::Bytes(b.into()),
            Value::SR_VALUE_THING(t) => sdbValue::Thing(t.into()),
        }
    }
}

impl Value {
    #[export_name = "sr_value_print"]
    pub extern "C" fn print_value(val: &Value) {
        println!("{val:?}");
    }

    #[export_name = "sr_value_eq"]
    pub extern "C" fn value_eq(lhs: &Value, rhs: &Value) -> bool {
        lhs == rhs
    }
}
