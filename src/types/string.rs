use std::{
    ffi::{c_char, CStr, CString},
    fmt::Debug,
    ptr,
};

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct string_t(pub *mut c_char);

impl string_t {
    pub fn null() -> string_t {
        string_t(ptr::null_mut())
    }
}

impl Debug for string_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = ptr_to_str(self.0);
        write!(f, "{}", str)
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

pub fn ptr_to_str(ptr: *const c_char) -> &'static str {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().unwrap()
}

#[no_mangle]
pub extern "C" fn free_string(string: string_t) {
    drop(string)
}
