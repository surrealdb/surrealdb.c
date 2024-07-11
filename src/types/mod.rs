use std::{
    ffi::{c_char, CStr},
    ptr::{self, slice_from_raw_parts_mut},
};

use surrealdb::sql;
use value::Value;

pub mod array;
pub mod bytes;
pub mod datetime;
pub mod duration;
pub mod notification;
pub mod object;
pub mod stream;
pub mod thing;
pub mod uuid;
pub mod value;

#[derive(Debug)]
#[repr(C)]
pub enum Number {
    Int(i64),
    Float(f64),
}

pub fn ptr_to_str(ptr: *const c_char) -> &'static str {
    let cstr = unsafe { CStr::from_ptr(ptr) };
    cstr.to_str().unwrap()
}
