pub mod array;
pub mod bytes;
pub mod duration;
pub mod notification;
pub mod object;
pub mod result;
pub mod stream;
pub mod string;
pub mod thing;
pub mod uuid;
pub mod value;

#[derive(Debug)]
#[repr(C)]
pub enum Number {
    Int(i64),
    Float(f64),
}
