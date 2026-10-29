use std::{
    ffi::{c_char, CStr, CString},
    fmt::{Debug, Display},
    ptr,
};

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

impl string_t {
    /// The contents as text, with invalid UTF-8 replaced. A null string reads as empty, so a
    /// caller-built value with a NULL string does not cause undefined behaviour.
    pub fn to_str_lossy(&self) -> std::borrow::Cow<'_, str> {
        if self.0.is_null() {
            return std::borrow::Cow::Borrowed("");
        }
        unsafe { CStr::from_ptr(self.0) }.to_string_lossy()
    }
}

impl Debug for string_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str_lossy())
    }
}

impl From<string_t> for String {
    fn from(value: string_t) -> Self {
        value.to_str_lossy().into_owned()
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
        match (self.0.is_null(), other.0.is_null()) {
            (true, true) => true,
            (false, false) => {
                let this = unsafe { CStr::from_ptr(self.0) };
                let that = unsafe { CStr::from_ptr(other.0) };
                this == that
            }
            _ => false,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_string_reads_as_empty() {
        let null = string_t::null();
        assert_eq!(null.to_str_lossy(), "");
        assert_eq!(format!("{null:?}"), "");
        assert_eq!(null, string_t::null());
        assert_ne!(null, string_t::from("x"));
        assert_eq!(String::from(null), "");
    }
}
