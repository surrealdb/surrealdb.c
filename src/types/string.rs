use std::{
    ffi::{c_char, CStr, CString},
    fmt::{Debug, Display},
    ptr,
};

use crate::utils::CStringExt2;

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct string_t(pub *mut c_char);

impl string_t {
    pub fn null() -> string_t {
        string_t(ptr::null_mut())
    }

    pub fn from_error(s: impl std::error::Error) -> string_t {
        s.to_string().to_string_t()
    }
}

impl Debug for string_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cstr = unsafe { CStr::from_ptr(self.0) };
        write!(f, "{}", cstr.to_string_lossy())
    }
}

impl From<string_t> for String {
    fn from(value: string_t) -> Self {
        let cstr = unsafe { CStr::from_ptr(value.0) };
        cstr.to_string_lossy().into()
    }
}

impl Drop for string_t {
    fn drop(&mut self) {
        let ptr = self.0;
        if !ptr.is_null() {
            let _ = unsafe { CString::from_raw(ptr) };
        }
    }
}

impl Clone for string_t {
    fn clone(&self) -> Self {
        let ptr = self.0;
        if ptr.is_null() {
            return string_t(ptr::null_mut());
        } else {
            let cstr = unsafe { CStr::from_ptr(ptr) };
            let cstring = CString::from(cstr);
            string_t(cstring.into_raw())
        }
    }
}

impl<D: Display> From<D> for string_t {
    fn from(value: D) -> Self {
        value.to_string().to_string_t()
    }
}

impl Default for string_t {
    fn default() -> Self {
        Self(ptr::null_mut())
    }
}

impl PartialEq for string_t {
    fn eq(&self, other: &Self) -> bool {
        // self.0 == other.0
        let self_cstr = unsafe { CStr::from_ptr(self.0) };
        let other_cstr = unsafe { CStr::from_ptr(other.0) };
        self_cstr == other_cstr
    }
}

// pub fn ptr_to_str(ptr: *const c_char) -> &'static str {
//     let cstr = unsafe { CStr::from_ptr(ptr) };
//     // // TODO(raphaeldarley): remove panic because of ub, or check its always caught
//     cstr.to_str().unwrap()
// }

#[export_name = "sr_free_string"]
pub extern "C" fn free_string(string: string_t) {
    drop(string)
}
