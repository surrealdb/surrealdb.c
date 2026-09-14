use surrealdb::types::File as sdbFile;

use crate::{string::string_t, utils::CStringExt2};

/// A reference to a file held in a bucket
///
/// Both fields are owned by this struct and are released by `sr_value_free`
/// when the file is reached through a `sr_value_t`.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    /// The bucket the file lives in
    pub bucket: string_t,
    /// The key identifying the file within the bucket
    pub key: string_t,
}

impl From<sdbFile> for File {
    fn from(value: sdbFile) -> Self {
        File {
            bucket: value.bucket.to_string_t(),
            key: value.key.to_string_t(),
        }
    }
}

impl From<File> for sdbFile {
    fn from(value: File) -> Self {
        sdbFile {
            bucket: String::from(value.bucket),
            key: String::from(value.key),
        }
    }
}
