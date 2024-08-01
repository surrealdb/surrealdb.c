use std::ffi::c_int;

use surrealdb::sql::Bytes as sdbBytes;

use super::array::{ArrayGen, MakeArray};

#[derive(Debug)]
#[repr(C)]
pub struct Bytes {
    pub arr: *mut u8,
    pub len: c_int,
}

impl Bytes {
    #[export_name = "sr_free_bytes"]
    pub extern "C" fn free_bytes(bytes: Bytes) {
        ArrayGen {
            ptr: bytes.arr,
            len: bytes.len,
        }
        .free()
    }
}

impl From<sdbBytes> for Bytes {
    fn from(value: sdbBytes) -> Self {
        let ArrayGen { ptr, len } = value.into_inner().make_array();
        Self { arr: ptr, len }
    }
}
