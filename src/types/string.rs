use std::ffi::{c_char, CStr, CString};
use std::fmt::{Debug, Display};
use std::ptr;

use crate::utils::CStringExt2;

/// A null-terminated C string type
///
/// This is a wrapper around a raw C string pointer that handles memory management.
/// Strings returned by SurrealDB functions must be freed with `sr_free_string`.
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
            string_t(ptr::null_mut())
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

/// Free a string allocated by SurrealDB
///
/// This function must be called to free strings returned by SurrealDB functions
/// to avoid memory leaks.
#[export_name = "sr_free_string"]
pub extern "C" fn free_string(string: string_t) {
    drop(string)
}
