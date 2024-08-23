use std::{
    collections::BTreeMap,
    ffi::{c_char, CStr},
};

use surrealdb::sql;

use crate::value::Value;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Object(Box<BTreeMap<String, Value>>);

impl Object {
    #[export_name = "sr_object_get"]
    pub extern "C" fn get(obj: &Object, key: *const c_char) -> Option<&Value> {
        let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap();
        obj.0.get(key)
    }

    pub extern "C" fn new() -> Object {
        let boxed = Box::new(BTreeMap::new());
        Object(boxed)
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
