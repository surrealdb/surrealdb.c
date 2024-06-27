use std::{
    ffi::{c_char, c_int, CString},
    fmt::Display,
    ptr::{self, slice_from_raw_parts_mut},
};

use crate::types;
use crate::{Array, Surreal};

/// when code = 0 there is no error
///
#[repr(C)]
pub struct SurrealError {
    code: c_int,
    msg: *mut c_char,
}

impl SurrealError {
    fn empty() -> Self {
        Self {
            code: 0,
            msg: ptr::null_mut(),
        }
    }

    fn from_msg(msg: impl Display) -> Self {
        Self {
            code: 1,
            msg: CString::new(msg.to_string()).unwrap().into_raw(),
        }
    }
}

#[repr(C)]
pub struct SurrealResult {
    pub ok: *mut Surreal,
    pub err: SurrealError,
}

impl SurrealResult {
    pub fn err(msg: impl Display) -> Self {
        Self {
            ok: ptr::null_mut(),
            err: SurrealError::from_msg(msg),
        }
    }
    pub fn ok(ok: &mut Surreal) -> Self {
        Self {
            ok,
            err: SurrealError::empty(),
        }
    }
}

#[repr(C)]
pub struct ArrayResult {
    pub ok: Array,
    pub err: SurrealError,
}

impl ArrayResult {
    pub fn err(msg: impl Display) -> Self {
        Self {
            ok: Array::empty(),
            err: SurrealError::from_msg(msg),
        }
    }
    pub fn ok(ok: Array) -> Self {
        Self {
            ok,
            err: SurrealError::empty(),
        }
    }
}

impl ArrayResult {
    #[no_mangle]
    pub extern "C" fn free_arr_res(res: ArrayResult) {
        if res.err.code != 0 {
            free_string(res.err.msg);
        } else {
            Array::free_arr(res.ok)
        }
    }
}

#[repr(C)]
pub struct ArrayResultArray {
    pub arr: *mut ArrayResult,
    pub len: usize,
}

impl From<Vec<ArrayResult>> for ArrayResultArray {
    fn from(value: Vec<ArrayResult>) -> Self {
        let boxed = value.into_boxed_slice();
        let slice = Box::leak(boxed);
        let len = slice.len();
        let pntr = std::ptr::from_mut(slice);
        Self {
            arr: pntr as *mut ArrayResult,
            len: len,
        }
    }
}

impl ArrayResultArray {
    #[no_mangle]
    pub extern "C" fn free_arr_res_arr(arr: ArrayResultArray) {
        let slice = slice_from_raw_parts_mut(arr.arr, arr.len);
        let boxed = unsafe { Box::from_raw(slice) };
        let owned = boxed.into_vec();
        for arr_res in owned {}
    }
}

#[repr(C)]
pub struct StringResult {
    pub ok: *mut c_char,
    pub err: SurrealError,
}

impl StringResult {
    pub fn err(msg: impl Display) -> Self {
        Self {
            ok: ptr::null_mut(),
            err: SurrealError::from_msg(msg),
        }
    }
    pub fn ok(ok: impl Into<String>) -> Self {
        Self {
            ok: CString::new(ok.into()).unwrap().into_raw(),
            err: SurrealError::empty(),
        }
    }
}

#[no_mangle]
pub extern "C" fn free_string(string: *mut c_char) {
    if !string.is_null() {
        let _ = unsafe { CString::from_raw(string) };
    }
}
