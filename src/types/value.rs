use surrealdb_core::sql;
use surrealdb_core::sql::Value as sdbValue;

pub use crate::{array::Array, object::Object, Number};
use crate::{bytes::Bytes, string::string_t, thing::Thing, utils::CStringExt2, uuid::Uuid};

use super::duration::Duration;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Default, Debug, Clone)]
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

impl Value {
    #[export_name = "sr_value_print"]
    pub extern "C" fn print_value(val: &Value) {
        println!("{val:?}");
    }
}
