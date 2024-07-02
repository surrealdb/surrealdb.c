use std::{
    collections::BTreeMap,
    ffi::{c_char, c_void, CStr},
};

use surrealdb::sql;

use crate::value::Value;

#[repr(C)]
#[derive(Debug)]
// pub struct Object(*mut c_void);
pub struct Object(*mut BTreeMap<String, Value>);

impl Object {
    #[no_mangle]
    pub extern "C" fn get(obj: &Object, key: *const c_char) -> Option<&Value> {
        let key = unsafe { CStr::from_ptr(key) }.to_str().unwrap();
        let inner = unsafe { &*obj.0 };
        inner.get(key)
    }
}

impl From<sql::Object> for Object {
    fn from(value: sql::Object) -> Self {
        let map = value.0;
        let mut new_map: BTreeMap<String, Value> =
            map.into_iter().map(|(k, v)| (k, v.into())).collect();
        // let pntr = std::ptr::from_mut(&mut new_map);
        let out = Self(&mut new_map);
        out
    }
}
