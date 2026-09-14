use std::ops::Bound as sdbBound;

use surrealdb::types::{Range as sdbRange, Value as sdbValue};

use crate::value::Value;

/// One end of a range
///
/// Mirrors `std::ops::Bound`: a bound is either absent, or present and either
/// inclusive or exclusive of its value.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum Bound {
    /// The range is open at this end
    SR_BOUND_UNBOUNDED,
    /// The range includes this value
    SR_BOUND_INCLUDED(Box<Value>),
    /// The range stops before this value
    SR_BOUND_EXCLUDED(Box<Value>),
}

impl From<sdbBound<sdbValue>> for Bound {
    fn from(value: sdbBound<sdbValue>) -> Self {
        match value {
            sdbBound::Unbounded => Bound::SR_BOUND_UNBOUNDED,
            sdbBound::Included(v) => Bound::SR_BOUND_INCLUDED(Box::new(Value::from(v))),
            sdbBound::Excluded(v) => Bound::SR_BOUND_EXCLUDED(Box::new(Value::from(v))),
        }
    }
}

impl From<Bound> for sdbBound<sdbValue> {
    fn from(value: Bound) -> Self {
        match value {
            Bound::SR_BOUND_UNBOUNDED => sdbBound::Unbounded,
            Bound::SR_BOUND_INCLUDED(v) => sdbBound::Included(sdbValue::from(*v)),
            Bound::SR_BOUND_EXCLUDED(v) => sdbBound::Excluded(sdbValue::from(*v)),
        }
    }
}

/// A range between two bounds
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    /// The lower bound
    pub start: Bound,
    /// The upper bound
    pub end: Bound,
}

impl From<sdbRange> for Range {
    fn from(value: sdbRange) -> Self {
        Range {
            start: Bound::from(value.start),
            end: Bound::from(value.end),
        }
    }
}

impl From<Range> for sdbRange {
    fn from(value: Range) -> Self {
        sdbRange {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}
