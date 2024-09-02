use std::{
    collections::BTreeMap,
    ffi::{c_char, c_double, c_float, c_int, CStr},
};

use surrealdb::sql;

use crate::{utils::CStringExt2, value::Value};

use crate::types::number::Number;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Object(Box<BTreeMap<String, Value>>);

impl Object {
    #[export_name = "sr_object_get"]
    pub extern "C" fn get(obj: &Object, key: *const c_char) -> Option<&Value> {
        let key = unsafe { CStr::from_ptr(key) }.to_string_lossy();
        obj.0.get(key.as_ref())
    }

    #[export_name = "sr_object_new"]
    pub extern "C" fn new() -> Object {
        let boxed = Box::new(BTreeMap::new());
        Object(boxed)
    }

    #[export_name = "sr_object_insert"]
    pub extern "C" fn insert(obj: *mut Object, key: *const c_char, value: &Value) {
        let obj = unsafe { &mut *obj };
        let key = unsafe { CStr::from_ptr(key) }.to_string_lossy().to_string();
        obj.0.insert(key, value.clone());
    }

    #[export_name = "sr_object_insert_str"]
    pub extern "C" fn insert_str(obj: *mut Object, key: *const c_char, value: *const c_char) {
        Self::insert(obj, key, &Value::SR_VALUE_STRAND(value.to_string_t()));
    }

    #[export_name = "sr_object_insert_int"]
    pub extern "C" fn insert_int(obj: *mut Object, key: *const c_char, value: c_int) {
        Self::insert(obj, key, &Value::SR_VALUE_NUMBER(Number::from(value)));
    }

    #[export_name = "sr_object_insert_float"]
    pub extern "C" fn insert_float(obj: *mut Object, key: *const c_char, value: c_float) {
        Self::insert(obj, key, &Value::SR_VALUE_NUMBER(Number::from(value)));
    }

    #[export_name = "sr_object_insert_double"]
    pub extern "C" fn insert_double(obj: *mut Object, key: *const c_char, value: c_double) {
        Self::insert(obj, key, &Value::SR_VALUE_NUMBER(Number::from(value)));
    }

    #[export_name = "sr_free_object"]
    pub extern "C" fn free_object(obj: Object) {
        drop(obj)
    }
}

impl From<sql::Object> for Object {
    fn from(value: sql::Object) -> Self {
        let map = value.0;
        let out = Self(Box::new(
            map.into_iter().map(|(k, v)| (k, v.into())).collect(),
        ));
        out
    }
}

impl From<&sql::Object> for Object {
    fn from(value: &sql::Object) -> Self {
        let map = &value.0;
        let out = Self(Box::new(
            map.iter().map(|(k, v)| (k.to_owned(), v.into())).collect(),
        ));
        out
    }
}
impl From<Object> for sql::Object {
    fn from(value: Object) -> Self {
        let map = value.0;
        let out: BTreeMap<String, sql::Value> =
            map.into_iter().map(|(k, v)| (k, v.into())).collect();
        out.into()
    }
}
