use std::ffi::{c_double, c_float, c_int};

use surrealdb::sql;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Number {
    SR_NUMBER_INT(i64),
    SR_NUMBER_FLOAT(f64),
}

impl From<c_int> for Number {
    fn from(value: c_int) -> Self {
        Number::SR_NUMBER_INT(value as i64)
    }
}

impl From<c_float> for Number {
    fn from(value: c_float) -> Self {
        Number::SR_NUMBER_FLOAT(value as f64)
    }
}

impl From<c_double> for Number {
    fn from(value: c_double) -> Self {
        Number::SR_NUMBER_FLOAT(value as f64)
    }
}

impl From<Number> for sql::Number {
    fn from(value: Number) -> Self {
        match value {
            Number::SR_NUMBER_INT(i) => sql::Number::Int(i),
            Number::SR_NUMBER_FLOAT(f) => sql::Number::Float(f),
        }
    }
}
