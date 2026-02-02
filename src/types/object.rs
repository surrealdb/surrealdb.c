use std::collections::BTreeMap;
use std::ffi::{c_char, c_double, c_float, c_int, CStr};

use surrealdb::sql;

use crate::types::number::Number;
use crate::utils::CStringExt2;
use crate::value::Value;

/// A key-value object type for SurrealDB
///
/// Contains string keys mapped to Value instances.
/// Uses Box to ensure a fixed size for FFI compatibility.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Object(Box<BTreeMap<String, Value>>);

impl Object {
    /// Get a value by key from the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid reference to an Object
    /// - `key` must be a valid null-terminated UTF-8 string
    #[export_name = "sr_object_get"]
    pub unsafe extern "C" fn get(obj: &Object, key: *const c_char) -> Option<&Value> {
        if key.is_null() {
            return None;
        }
        let key = unsafe { CStr::from_ptr(key) }.to_string_lossy();
        obj.0.get(key.as_ref())
    }

    /// Create a new empty object
    #[export_name = "sr_object_new"]
    pub extern "C" fn new() -> Self {
        Object(Box::new(BTreeMap::new()))
    }

    /// Insert a key-value pair into the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    /// - `key` must be a valid null-terminated UTF-8 string
    /// - `value` must be a valid reference to a Value
    ///
    /// If any pointer is null, the function returns without modification.
    #[export_name = "sr_object_insert"]
    pub unsafe extern "C" fn insert(obj: *mut Object, key: *const c_char, value: &Value) {
        if obj.is_null() || key.is_null() {
            return;
        }
        let obj = unsafe { &mut *obj };
        let key = unsafe { CStr::from_ptr(key) }.to_string_lossy().to_string();
        obj.0.insert(key, value.clone());
    }

    /// Insert a string value into the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    /// - `key` must be a valid null-terminated UTF-8 string
    /// - `value` must be a valid null-terminated UTF-8 string
    #[export_name = "sr_object_insert_str"]
    pub unsafe extern "C" fn insert_str(
        obj: *mut Object,
        key: *const c_char,
        value: *const c_char,
    ) {
        if obj.is_null() || key.is_null() || value.is_null() {
            return;
        }
        Self::insert(obj, key, &Value::SR_VALUE_STRAND(value.to_string_t()));
    }

    /// Insert an integer value into the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    /// - `key` must be a valid null-terminated UTF-8 string
    #[export_name = "sr_object_insert_int"]
    pub unsafe extern "C" fn insert_int(obj: *mut Object, key: *const c_char, value: c_int) {
        if obj.is_null() || key.is_null() {
            return;
        }
        Self::insert(obj, key, &Value::SR_VALUE_NUMBER(Number::from(value)));
    }

    /// Insert a float value into the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    /// - `key` must be a valid null-terminated UTF-8 string
    #[export_name = "sr_object_insert_float"]
    pub unsafe extern "C" fn insert_float(obj: *mut Object, key: *const c_char, value: c_float) {
        if obj.is_null() || key.is_null() {
            return;
        }
        Self::insert(obj, key, &Value::SR_VALUE_NUMBER(Number::from(value)));
    }

    /// Insert a double value into the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    /// - `key` must be a valid null-terminated UTF-8 string
    #[export_name = "sr_object_insert_double"]
    pub unsafe extern "C" fn insert_double(obj: *mut Object, key: *const c_char, value: c_double) {
        if obj.is_null() || key.is_null() {
            return;
        }
        Self::insert(obj, key, &Value::SR_VALUE_NUMBER(Number::from(value)));
    }

    /// Free an object
    #[export_name = "sr_free_object"]
    pub extern "C" fn free_object(obj: Object) {
        drop(obj)
    }

    /// Get the number of key-value pairs in the object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    #[export_name = "sr_object_len"]
    pub unsafe extern "C" fn object_len(obj: *const Object) -> c_int {
        if obj.is_null() {
            return 0;
        }
        unsafe { (*obj).0.len() as c_int }
    }

    /// Get all keys from the object as a null-terminated array of strings
    /// Returns the number of keys, or -1 on error
    /// The caller must free the returned array using sr_free_string_arr
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    /// - `keys_ptr` must be a valid pointer to receive the keys
    #[export_name = "sr_object_keys"]
    pub unsafe extern "C" fn object_keys(
        obj: *const Object,
        keys_ptr: *mut *mut *mut c_char,
    ) -> c_int {
        if obj.is_null() || keys_ptr.is_null() {
            return -1;
        }

        let object = unsafe { &*obj };
        let keys: Vec<*mut c_char> = object
            .0
            .keys()
            .map(|k| {
                let cstring = std::ffi::CString::new(k.as_str()).unwrap_or_default();
                cstring.into_raw()
            })
            .collect();

        let len = keys.len() as c_int;

        if len == 0 {
            unsafe {
                *keys_ptr = std::ptr::null_mut();
            }
            return 0;
        }

        // Allocate array for the pointers
        let boxed = keys.into_boxed_slice();
        let ptr = Box::into_raw(boxed) as *mut *mut c_char;
        unsafe {
            *keys_ptr = ptr;
        }

        len
    }

    /// Free a string array returned by sr_object_keys
    ///
    /// # Safety
    ///
    /// - `arr` must be a valid pointer to an array
    /// - `len` must be the length of the array
    #[export_name = "sr_free_string_arr"]
    pub unsafe extern "C" fn free_string_arr(arr: *mut *mut c_char, len: c_int) {
        if arr.is_null() || len <= 0 {
            return;
        }

        let slice = unsafe { std::slice::from_raw_parts_mut(arr, len as usize) };
        for ptr in slice.iter_mut() {
            if !ptr.is_null() {
                let _ = unsafe { std::ffi::CString::from_raw(*ptr) };
            }
        }

        // Free the array itself
        let _ = unsafe { Box::from_raw(core::ptr::slice_from_raw_parts_mut(arr, len as usize)) };
    }
}

impl From<sql::Object> for Object {
    fn from(value: sql::Object) -> Self {
        let map = value.0;
        Self(Box::new(map.into_iter().map(|(k, v)| (k, v.into())).collect()))
    }
}

impl From<&sql::Object> for Object {
    fn from(value: &sql::Object) -> Self {
        let map = &value.0;
        Self(Box::new(map.iter().map(|(k, v)| (k.to_owned(), v.into())).collect()))
    }
}
impl From<Object> for sql::Object {
    fn from(value: Object) -> Self {
        let map = *value.0;
        let out: BTreeMap<String, sql::Value> =
            map.into_iter().map(|(k, v)| (k, v.into())).collect();
        out.into()
    }
}
