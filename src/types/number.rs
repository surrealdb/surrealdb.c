use std::ffi::{c_double, c_float, c_int, CStr};

use surrealdb::types::Number as sdbNumber;
use rust_decimal::Decimal;
use crate::string::string_t;
use crate::utils::CStringExt2;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum Number {
    SR_NUMBER_INT(i64),
    SR_NUMBER_FLOAT(f64),
    /// Decimal stored as string representation for C compatibility
    SR_NUMBER_DECIMAL(string_t),
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

impl From<Decimal> for Number {
    fn from(value: Decimal) -> Self {
        Number::SR_NUMBER_DECIMAL(value.to_string().to_string_t())
    }
}

impl From<Number> for sdbNumber {
    fn from(value: Number) -> Self {
        match value {
            Number::SR_NUMBER_INT(i) => sdbNumber::Int(i),
            Number::SR_NUMBER_FLOAT(f) => sdbNumber::Float(f),
            Number::SR_NUMBER_DECIMAL(s) => {
                let cstr = unsafe { CStr::from_ptr(s.0) };
                let decimal_str = cstr.to_string_lossy();
                match decimal_str.parse::<Decimal>() {
                    Ok(d) => sdbNumber::Decimal(d),
                    // Fallback to float if parsing fails
                    Err(_) => sdbNumber::Float(decimal_str.parse().unwrap_or(0.0)),
                }
            }
        }
    }
}
