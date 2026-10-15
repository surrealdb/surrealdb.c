use std::{ffi::c_int, mem::ManuallyDrop, ptr::slice_from_raw_parts};

use surrealdb::types::Bytes as sdbBytes;

use super::array::{ArrayGen, MakeArray};

/// An owned byte buffer allocated by SurrealDB
///
/// A `Bytes` owns its buffer: dropping it (or passing it to `sr_free_bytes`) frees the buffer,
/// and cloning it copies the buffer.
#[derive(Debug)]
#[repr(C)]
pub struct Bytes {
    pub arr: *mut u8,
    pub len: c_int,
}

impl Bytes {
    pub fn empty() -> Self {
        Self {
            arr: std::ptr::null_mut(),
            len: 0,
        }
    }

    pub fn as_slice<'a>(&'a self) -> &'a [u8] {
        if self.arr.is_null() || self.len <= 0 {
            return &[];
        }
        let slice = slice_from_raw_parts(self.arr, self.len as usize);
        unsafe { &*slice }
    }

    /// Free a bytes value returned by SurrealDB
    ///
    /// The bytes must not be used after this call. Do not call this on the bytes inside an
    /// `sr_value_t`; those are freed with the value.
    #[export_name = "sr_free_bytes"]
    pub extern "C" fn free_bytes(bytes: Bytes) {
        drop(bytes)
    }

    /// Free a byte buffer returned by the RPC API (`sr_surreal_rpc_execute`, `sr_rpc_stream_next`)
    ///
    /// `len` must be the length returned alongside the buffer.
    #[export_name = "sr_free_byte_arr"]
    pub extern "C" fn free_byte_arr(ptr: *mut u8, len: c_int) {
        ArrayGen { ptr, len }.free()
    }
}

impl Drop for Bytes {
    fn drop(&mut self) {
        ArrayGen {
            ptr: self.arr,
            len: self.len,
        }
        .free()
    }
}

impl PartialEq for Bytes {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Clone for Bytes {
    fn clone(&self) -> Self {
        if self.as_slice().is_empty() {
            return Self::empty();
        }
        self.as_slice().to_vec().make_array().into()
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
        // Ownership of the buffer moves to the ArrayGen, so don't run Bytes' destructor
        let value = ManuallyDrop::new(value);
        Self {
            ptr: value.arr,
            len: value.len,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_is_deep() {
        let original: Bytes = vec![1u8, 2, 3].make_array().into();
        let copy = original.clone();
        assert_ne!(original.arr, copy.arr);
        assert_eq!(original, copy);

        // Consuming the copy must leave the original intact
        let converted: sdbBytes = copy.into();
        assert_eq!(converted.as_ref(), &[1, 2, 3]);
        assert_eq!(original.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn empty_clone_and_drop() {
        let empty = Bytes::empty();
        let copy = empty.clone();
        assert!(copy.arr.is_null());
        drop(empty);
        drop(copy);
    }
}
