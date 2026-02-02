use std::ffi::{CStr, CString};

use libc::c_char;

use crate::string::string_t;

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
        // Replace null bytes with empty string to avoid panic
        let cstring = CString::new(self).unwrap_or_else(|_| CString::new("").unwrap());
        cstring.into_raw()
    }
}

impl CStringExt for &str {
    fn to_raw_char_ptr(self) -> *mut c_char {
        // Replace null bytes with empty string to avoid panic
        let cstring = CString::new(self).unwrap_or_else(|_| CString::new("").unwrap());
        cstring.into_raw()
    }
}

impl CStringExt for *const c_char {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn to_raw_char_ptr(self) -> *mut c_char {
        let cstr = unsafe { CStr::from_ptr(self) };
        let cstring = CString::from(cstr);
        cstring.into_raw()
    }
}
