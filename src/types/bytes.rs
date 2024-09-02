use std::{ffi::c_int, ptr::slice_from_raw_parts};

use surrealdb::sql::Bytes as sdbBytes;

use super::array::{ArrayGen, MakeArray};

#[derive(Debug)]
#[repr(C)]
pub struct Bytes {
    pub arr: *mut u8,
    pub len: c_int,
}

impl Bytes {
    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        if self.arr.is_null() || self.len == 0 {
            return &[];
        }
        let slice = slice_from_raw_parts(self.arr, self.len as usize);
        unsafe { &*slice }
    }

    #[export_name = "sr_free_bytes"]
    pub extern "C" fn free_bytes(bytes: Bytes) {
        ArrayGen {
            ptr: bytes.arr,
            len: bytes.len,
        }
        .free()
    }

    #[export_name = "sr_free_byte_arr"]
    pub extern "C" fn free_byte_arr(ptr: *mut u8, len: c_int) {
        ArrayGen { ptr: ptr, len: len }.free()
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
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
