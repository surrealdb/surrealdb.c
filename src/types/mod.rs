use std::ptr::{self, slice_from_raw_parts_mut};

use surrealdb::sql;
use value::Value;

pub mod array;
pub mod duration;
pub mod object;
pub mod value;

#[derive(Debug)]
#[repr(C)]
pub enum Number {
    Int(i64),
    Float(f64),
}
