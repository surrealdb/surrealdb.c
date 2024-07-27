use std::{ffi::CString, ptr};

use libc::c_char;

use crate::{result::ArrayResultArray, string::string_t, value::Array};

pub trait CStringExt {
    fn to_raw_char_ptr(self) -> *mut c_char;
}

pub trait CStringExt2 {
    fn to_string_t(self) -> string_t;
}

impl<T> CStringExt2 for T
where
    T: CStringExt,
{
    fn to_string_t(self) -> string_t {
        string_t(self.to_raw_char_ptr())
    }
}

impl CStringExt for String {
    fn to_raw_char_ptr(self) -> *mut c_char {
        let cstring = CString::new(self).unwrap();
        cstring.into_raw()
    }
}

impl CStringExt for &str {
    fn to_raw_char_ptr(self) -> *mut c_char {
        let cstring = CString::new(self).unwrap();
        cstring.into_raw()
    }
}

// TODO Default could be used instead
pub trait Empty {
    fn empty() -> Self;
}

impl Empty for () {
    fn empty() -> Self {
        ()
    }
}

impl<T> Empty for *mut T {
    fn empty() -> Self {
        ptr::null_mut()
    }
}

impl Empty for Array {
    fn empty() -> Self {
        Array::empty()
    }
}

impl Empty for ArrayResultArray {
    fn empty() -> Self {
        ArrayResultArray::empty()
    }
}

impl Empty for string_t {
    fn empty() -> Self {
        string_t::null()
    }
}
