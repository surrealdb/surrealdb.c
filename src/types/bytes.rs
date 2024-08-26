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

impl Clone for Bytes {
    fn clone(&self) -> Self {
        Self {
            arr: self.arr.clone(),
            len: self.len.clone(),
        }
    }
}

impl From<ArrayGen<u8>> for Bytes {
    fn from(value: ArrayGen<u8>) -> Self {
        let ArrayGen { ptr, len } = value;
        Self { arr: ptr, len }
    }
}

impl From<Bytes> for ArrayGen<u8> {
    fn from(value: Bytes) -> Self {
        let Bytes { arr, len } = value;
        Self { ptr: arr, len }
    }
}

impl From<sdbBytes> for Bytes {
    fn from(value: sdbBytes) -> Self {
        value.into_inner().make_array().into()
    }
}

impl From<Bytes> for sdbBytes {
    fn from(value: Bytes) -> Self {
        ArrayGen::from(value).into_vec().into()
    }
}
