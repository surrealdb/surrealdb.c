use std::{
    ffi::c_int,
    fmt::Display,
    ptr::{self, slice_from_raw_parts_mut},
};

use crate::{string::free_string, utils::CStringExt2};
use crate::{string::string_t, Array, Surreal};

/// when code = 0 there is no error
///
#[repr(C)]
pub struct SurrealError {
    code: c_int,
    msg: string_t,
}

impl SurrealError {
    pub fn empty() -> Self {
        Self {
            code: 0,
            msg: string_t::null(),
        }
    }

    pub fn from_msg(msg: impl Display) -> Self {
        let out = Self {
            code: 1,
            msg: msg.to_string().to_string_t(),
        };
        out
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
    fn empty() -> Self {
        Self {
            arr: ptr::null_mut(),
            len: 0,
        }
    }
}

impl ArrayResultArray {
    #[no_mangle]
    pub extern "C" fn free_arr_res_arr(arr: ArrayResultArray) {
        let slice = slice_from_raw_parts_mut(arr.arr, arr.len);
        let boxed = unsafe { Box::from_raw(slice) };
        let owned = boxed.into_vec();
        for arr_res in owned {
            ArrayResult::free_arr_res(arr_res)
        }
    }
}

#[repr(C)]
pub struct ArrayResultArrayResult {
    pub ok: ArrayResultArray,
    pub err: SurrealError,
}

impl ArrayResultArrayResult {
    pub fn err(msg: impl Display) -> Self {
        Self {
            ok: ArrayResultArray::empty(),
            err: SurrealError::from_msg(msg),
        }
    }
    pub fn ok(ok: ArrayResultArray) -> Self {
        Self {
            ok,
            err: SurrealError::empty(),
        }
    }
}

impl ArrayResultArrayResult {
    #[no_mangle]
    pub extern "C" fn free_arr_res_arr_res(res: ArrayResultArrayResult) {
        if res.err.code != 0 {
            free_string(res.err.msg);
        } else {
            ArrayResultArray::free_arr_res_arr(res.ok)
        }
    }
}

#[repr(C)]
pub struct StringResult {
    pub ok: string_t,
    pub err: SurrealError,
}

impl StringResult {
    pub fn err(msg: impl Display) -> Self {
        Self {
            ok: string_t::null(),
            err: SurrealError::from_msg(msg),
        }
    }
    pub fn ok(ok: impl Into<String>) -> Self {
        Self {
            ok: ok.into().to_string_t(),
            err: SurrealError::empty(),
        }
    }
}
