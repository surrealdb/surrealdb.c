use std::ptr::{self, slice_from_raw_parts_mut};

use crate::value::Value;
use surrealdb::sql;

#[derive(Debug)]
#[repr(C)]
pub struct Array {
    arr: *mut Value,
    len: usize,
}

impl From<sql::Array> for Array {
    fn from(value: sql::Array) -> Self {
        let val_vec: Vec<Value> = value.0.into_iter().map(Into::into).collect();
        val_vec.into()
    }
}

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        let boxed = value.into_boxed_slice();
        let slice = Box::leak(boxed);
        let len = slice.len();
        let pntr = std::ptr::from_mut(slice);
        Self {
            arr: pntr as *mut Value,
            len: len,
        }
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

impl Array {
    #[no_mangle]
    pub extern "C" fn free_arr(arr: Array) {
        let slice = slice_from_raw_parts_mut(arr.arr, arr.len);
        let _boxed = unsafe { Box::from_raw(slice) };
    }
}
