use std::{ffi::c_int, mem::ManuallyDrop, ptr::slice_from_raw_parts};

use surrealdb::types::Bytes as sdbBytes;

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

    #[export_name = "sr_byte_arr_free"]
    pub extern "C" fn byte_arr_free(ptr: *mut u8, len: c_int) {
        ArrayGen { ptr, len }.free()
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Clone for Bytes {
    /// Deep. The obvious field-wise clone copies `arr` as a *pointer*, giving
    /// two Bytes that alias one buffer -- harmless while nothing freed it, and
    /// a double free the moment `Drop` exists. `sr_object_insert` clones every
    /// value it is handed, so this path is hot.
    fn clone(&self) -> Self {
        let slice = self.as_slice();
        if slice.is_empty() {
            return Self { arr: std::ptr::null_mut(), len: 0 };
        }
        slice.to_vec().make_array().into()
    }
}

impl Drop for Bytes {
    /// Without this, `sr_value_free` on a BYTES value reclaimed the boxed Value
    /// and orphaned its payload: drop glue walks fields, and a raw `*mut u8`
    /// has nothing to run.
    fn drop(&mut self) {
        ArrayGen {
            ptr: self.arr,
            len: self.len,
        }
        .free();
        self.arr = std::ptr::null_mut();
        self.len = 0;
    }
}

impl From<ArrayGen<u8>> for Bytes {
    fn from(value: ArrayGen<u8>) -> Self {
        let ArrayGen { ptr, len } = value;
        Self { arr: ptr, len }
    }
}

impl From<Bytes> for ArrayGen<u8> {
    /// Hands the buffer over. `ManuallyDrop` because `Bytes` now has a
    /// destructor, so the fields cannot simply be moved out -- and running it
    /// here would free the very allocation being transferred.
    fn from(value: Bytes) -> Self {
        let value = ManuallyDrop::new(value);
        Self { ptr: value.arr, len: value.len }
    }
}

impl From<sdbBytes> for Bytes {
    fn from(value: sdbBytes) -> Self {
        let vec: Vec<u8> = value.as_ref().to_vec();
        vec.make_array().into()
    }
}

impl From<Bytes> for sdbBytes {
    fn from(value: Bytes) -> Self {
        let vec = ArrayGen::from(value).into_vec();
        sdbBytes::from(vec)
    }
}
