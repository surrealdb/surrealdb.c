use std::ffi::{c_char, CString};

use surrealdb_core::sql;
use surrealdb_core::sql::Value as sdbValue;

pub use crate::{array::Array, object::Object, Number};
use crate::{bytes::Bytes, thing::Thing, uuid::Uuid};

use super::duration::Duration;

#[repr(C)]
#[derive(Default, Debug)]
pub enum Value {
    #[default]
    None,
    Null,
    Bool(bool),
    Number(Number),
    Strand(*mut c_char),
    Duration(Duration),
    // Datetime(Datetime),
    Uuid(Uuid),
    Array(Box<Array>),
    Object(Object),
    // Geometry(Geometry),
    Bytes(Bytes),
    Thing(Thing),
}

impl From<sdbValue> for Value {
    fn from(value: sdbValue) -> Self {
        match value {
            sdbValue::None => Value::None,
            sdbValue::Null => Value::Null,
            sdbValue::Bool(b) => Value::Bool(b),
            sdbValue::Number(n) => match n {
                sql::Number::Int(i) => Value::Number(Number::Int(i)),
                sql::Number::Float(f) => Value::Number(Number::Float(f)),
                sql::Number::Decimal(_) => todo!(),
                _ => todo!(),
            },
            sdbValue::Strand(s) => Value::Strand(CString::new(s.0).unwrap().into_raw()),
            sdbValue::Duration(d) => Value::Duration(d.into()),
            sdbValue::Datetime(_) => todo!(),
            sdbValue::Uuid(u) => Value::Uuid(u.into()),
            // unecssary box see: https://github.com/mozilla/cbindgen/issues/981
            sdbValue::Array(a) => Value::Array(Box::new(a.into())),
            sdbValue::Object(o) => Value::Object(o.into()),
            sdbValue::Geometry(_) => todo!(),
            sdbValue::Bytes(b) => Value::Bytes(b.into()),
            sdbValue::Thing(t) => Value::Thing(t.into()),
            _ => unimplemented!("other variants shouldn't be returned"),
        }
    }
}

impl Value {
    #[no_mangle]
    pub extern "C" fn print_value(val: &Value) {
        println!("{val:?}");
    }
}
