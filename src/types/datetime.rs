use libc::time_t;
use libc::tm;

use crate::duration::Duration;

#[repr(C)]
#[derive(Debug)]
pub struct DateTime {
    /// ISO 8601, 0 is 1 BC
    date: u32,
    time: Duration,
}

// impl<T> From<chrono::DateTime<T>> for DateTime {
//     fn from(value: chrono::DateTime<T>) -> Self {
//         todo!()
//     }
// }
