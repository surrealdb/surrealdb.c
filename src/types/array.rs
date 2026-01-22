use std::{
    ffi::c_int,
    ptr::{self, slice_from_raw_parts, slice_from_raw_parts_mut},
};

use surrealdb::sql;
use super::value::Value;

#[repr(C)]
pub struct ArrayGen<T> {
    pub ptr: *mut T,
    pub len: c_int,
}

impl<T: std::fmt::Debug> std::fmt::Debug for ArrayGen<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

impl<T: PartialEq> PartialEq for ArrayGen<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T> Clone for ArrayGen<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        if self.ptr.is_null() || self.len == 0 {
            return Self {
                ptr: ptr::null_mut(),
                len: 0,
            };
        }
        let slice = unsafe { &*slice_from_raw_parts(self.ptr, self.len as usize) };
        slice.to_owned().make_array()
    }
}
impl<T> ArrayGen<T> {
    pub fn into_vec(self) -> Vec<T> {
        if self.ptr.is_null() || self.len == 0 {
            return Vec::with_capacity(0);
        }
        let slice = slice_from_raw_parts_mut(self.ptr, self.len as usize);
        let boxed = unsafe { Box::from_raw(slice) };
        boxed.into_vec()
    }

    pub fn as_slice<'a>(&'a self) -> &'a [T] {
        if self.ptr.is_null() || self.len == 0 {
            return &[];
        }
        let slice = slice_from_raw_parts(self.ptr, self.len as usize);
        unsafe { &*slice }
    }
}

impl<T> ArrayGen<T> {
    pub fn free(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        if self.len <= 0 {
            return;
        }
        let slice = slice_from_raw_parts_mut(self.ptr, self.len as usize);
        let _boxed = unsafe { Box::from_raw(slice) };
    }
}

pub trait MakeArray<T> {
    fn make_array(self) -> ArrayGen<T>;
}

impl<T> MakeArray<T> for Vec<T> {
    fn make_array(self) -> ArrayGen<T> {
        let boxed = self.into_boxed_slice();
        let slice = Box::leak(boxed);
        let len = slice.len();
        let pntr = std::ptr::from_mut(slice);
        ArrayGen {
            ptr: pntr as *mut T,
            len: len as c_int,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Array {
    pub arr: *mut Value,
    pub len: c_int,
}

impl From<sql::Array> for Array {
    fn from(value: sql::Array) -> Self {
        let val_vec: Vec<Value> = value.0.into_iter().map(Into::into).collect();
        val_vec.into()
    }
}

impl From<&sql::Array> for Array {
    fn from(value: &sql::Array) -> Self {
        let val_vec: Vec<Value> = value.0.iter().map(Into::into).collect();
        val_vec.into()
    }
}

impl From<Array> for sql::Array {
    fn from(value: Array) -> Self {
        let gen_arr: ArrayGen<Value> = value.into();
        let vec: Vec<sql::Value> = gen_arr.into_vec().into_iter().map(Into::into).collect();

        Self::from(vec)
    }
}

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        value.make_array().into()
    }
}

impl From<ArrayGen<Value>> for Array {
    fn from(value: ArrayGen<Value>) -> Self {
        let ArrayGen { ptr, len } = value;
        Self { arr: ptr, len }
    }
}

impl From<Array> for ArrayGen<Value> {
    fn from(value: Array) -> Self {
        let result = ArrayGen { ptr: value.arr, len: value.len };
        // Prevent Array's Drop from running to avoid double-free
        std::mem::forget(value);
        result
    }
}

impl From<&Array> for ArrayGen<Value> {
    fn from(value: &Array) -> Self {
        let Array { arr, len } = *value;
        ArrayGen { ptr: arr, len }
    }
}

impl Array {
    pub fn empty() -> Self {
        Self {
            arr: ptr::null_mut(),
            len: 0,
        }
    }

    pub fn as_slice<'a>(&'a self) -> &'a [Value] {
        if self.arr.is_null() || self.len == 0 {
            return &[];
        }
        let slice = slice_from_raw_parts(self.arr, self.len as usize);
        unsafe { &*slice }
    }
}

impl Clone for Array {
    fn clone(&self) -> Self {
        ArrayGen {
            ptr: self.arr,
            len: self.len,
        }
        .clone()
        .into()
    }
}

impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Drop for Array {
    fn drop(&mut self) {
        ArrayGen {
            ptr: self.arr,
            len: self.len,
        }
        .free()
    }
}

impl Array {
    #[export_name = "sr_free_arr"]
    pub extern "C" fn free_arr(ptr: *mut Value, len: c_int) {
        ArrayGen { ptr, len }.free()
    }

    /// Get the length of an array
    #[export_name = "sr_array_len"]
    pub extern "C" fn array_len(arr: *const Array) -> c_int {
        if arr.is_null() {
            return 0;
        }
        unsafe { (*arr).len }
    }

    /// Get a value at the specified index (returns NULL if out of bounds)
    /// The returned pointer is borrowed and should NOT be freed by the caller
    #[export_name = "sr_array_get"]
    pub extern "C" fn array_get(arr: *const Array, index: c_int) -> *const Value {
        if arr.is_null() {
            return ptr::null();
        }
        let array = unsafe { &*arr };
        if index < 0 || index >= array.len {
            return ptr::null();
        }
        unsafe { array.arr.add(index as usize) }
    }

    /// Create a new array with the given value appended
    /// Returns a new array - the original array is not modified
    /// The caller is responsible for freeing the returned array
    #[export_name = "sr_array_push"]
    pub extern "C" fn array_push(arr: *const Array, value: *const Value) -> *mut Array {
        if value.is_null() {
            // If value is null, return a copy of the original or empty array
            if arr.is_null() {
                return Box::into_raw(Box::new(Array::empty()));
            }
            let array = unsafe { &*arr };
            return Box::into_raw(Box::new(array.clone()));
        }
        
        let value = unsafe { (*value).clone() };
        
        if arr.is_null() {
            // Create new array with just this value
            let vec = vec![value];
            return Box::into_raw(Box::new(vec.into()));
        }
        
        let array = unsafe { &*arr };
        let mut vec: Vec<Value> = array.as_slice().to_vec();
        vec.push(value);
        Box::into_raw(Box::new(vec.into()))
    }

    /// Free an array created by sr_array_push
    #[export_name = "sr_array_free"]
    pub extern "C" fn array_free(arr: *mut Array) {
        if !arr.is_null() {
            let _ = unsafe { Box::from_raw(arr) };
        }
    }
}
