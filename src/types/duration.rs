use surrealdb::types::Duration as sdbDuration;

#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct Duration {
    pub secs: u64,
    pub nanos: u32,
}

impl From<std::time::Duration> for Duration {
    fn from(value: std::time::Duration) -> Self {
        Self {
            secs: value.as_secs(),
            nanos: value.subsec_nanos(),
        }
    }
}

impl From<sdbDuration> for Duration {
    fn from(value: sdbDuration) -> Self {
        let std_dur: std::time::Duration = value.into();
        std_dur.into()
    }
}

impl From<Duration> for sdbDuration {
    fn from(value: Duration) -> Self {
        std::time::Duration::new(value.secs, value.nanos).into()
    }
}
