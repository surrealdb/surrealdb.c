use std::{
    ffi::c_int,
    ptr::{self, slice_from_raw_parts_mut},
};

use crate::value::Value;
use surrealdb::sql;

pub struct ArrayGen<T> {
    pub ptr: *mut T,
    pub len: c_int,
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

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        let ArrayGen { ptr, len } = value.make_array();
        Self { arr: ptr, len: len }
    }
}

impl Array {
    pub fn empty() -> Self {
        Self {
            arr: ptr::null_mut(),
            len: 0,
        }
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
}
