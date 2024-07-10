use surrealdb::sql::Bytes as sdbBytes;

#[derive(Debug)]
#[repr(C)]
pub struct Bytes {
    pub arr: *mut u8,
    pub len: usize,
}

impl From<sdbBytes> for Bytes {
    fn from(value: sdbBytes) -> Self {
        let boxed = value.into_inner().into_boxed_slice();
        let slice = Box::leak(boxed);
        let len = slice.len();
        let pntr = std::ptr::from_mut(slice);
        Self {
            arr: pntr as *mut u8,
            len: len,
        }
    }
}
