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

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
#[repr(C)]
pub enum Number {
    SR_NUMBER_INT(i64),
    SR_NUMBER_FLOAT(f64),
}
