use std::{
    collections::BTreeMap,
    ffi::{c_char, c_double, c_float, c_int, CStr},
};

use surrealdb::sql;

use crate::{utils::CStringExt2, value::Value};

use crate::types::number::Number;

use super::{array::MakeArray, string::string_t, thing::Thing};

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

    #[export_name = "sr_object_insert_thing"]
    pub extern "C" fn insert_thing(obj: *mut Object, key: *const c_char, value: Thing) {
        Self::insert(obj, key, &Value::SR_VALUE_THING(value.clone()));
    }

    #[export_name = "sr_object_into_arr"]
    pub extern "C" fn into_arr(
        obj: Object,
        key_ptr: *mut *mut string_t,
        val_ptr: *mut *mut Value,
    ) -> c_int {
        let mut key_vec = Vec::new();
        let mut val_vec = Vec::new();
        for (k, v) in *obj.0 {
            key_vec.push(k.to_string_t());
            val_vec.push(v);
        }

        let key_arr = key_vec.make_array();
        unsafe { key_ptr.write(key_arr.ptr) }

        let val_arr = val_vec.make_array();
        unsafe { val_ptr.write(val_arr.ptr) }

        key_arr.len
    }

    #[export_name = "sr_free_object"]
    pub extern "C" fn free_object(obj: Object) {
        drop(obj)
    }

    #[export_name = "sr_object_eq"]
    pub extern "C" fn object_eq(lhs: &Object, rhs: &Object) -> bool {
        lhs == rhs
    }

    #[export_name = "sr_object_print"]
    pub extern "C" fn object_print(value: &Object) {
        println!("{value:?}");
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

impl From<&Object> for sql::Object {
    fn from(value: &Object) -> Self {
        let map = &value.0;
        let out: BTreeMap<String, sql::Value> =
            map.iter().map(|(k, v)| (k.to_owned(), v.into())).collect();
        out.into()
    }
}
