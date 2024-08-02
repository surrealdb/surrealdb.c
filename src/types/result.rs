use std::{ffi::c_int, fmt::Display};

use crate::string::string_t;
use crate::{utils::CStringExt2, SR_ERROR};

use super::array::{Array, ArrayGen};

/// when code = 0 there is no error
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
            code: SR_ERROR,
            msg: msg.to_string().to_string_t(),
        };
        out
    }
}

// #[repr(C)]
// pub struct SurrealResult {
//     pub ok: *mut Surreal,
//     pub err: SurrealError,
// }

// impl SurrealResult {
//     pub fn err(msg: impl Display) -> Self {
//         Self {
//             ok: ptr::null_mut(),
//             err: SurrealError::from_msg(msg),
//         }
//     }
//     pub fn ok(ok: &mut Surreal) -> Self {
//         Self {
//             ok,
//             err: SurrealError::empty(),
//         }
//     }
// }

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
    #[export_name = "sr_free_arr_res"]
    pub extern "C" fn free_arr_res(res: ArrayResult) {
        let _ = res;
    }
}

#[export_name = "sr_free_arr_res_arr"]
pub extern "C" fn free_arr_res_arr(ptr: *mut ArrayResult, len: c_int) {
    ArrayGen { ptr, len }.free()
}
