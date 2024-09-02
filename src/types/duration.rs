use surrealdb::sql;

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Duration {
    secs: u64,
    nanos: u32,
}

impl From<std::time::Duration> for Duration {
    fn from(value: std::time::Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl From<surrealdb::sql::Duration> for Duration {
    fn from(value: surrealdb::sql::Duration) -> Self {
        value.0.into()
    }
}

impl From<Duration> for sql::Duration {
    fn from(value: Duration) -> Self {
        std::time::Duration::new(value.secs, value.nanos).into()
    }
}
