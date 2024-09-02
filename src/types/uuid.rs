use surrealdb::sql::Uuid as sdbUuid;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Uuid(pub [u8; 16]);

impl From<sdbUuid> for Uuid {
    fn from(value: sdbUuid) -> Self {
        let bytes = value.0.into_bytes();
        Self(bytes)
    }
}

impl From<uuid::Uuid> for Uuid {
    fn from(value: uuid::Uuid) -> Self {
        Self(value.into_bytes())
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(value: Uuid) -> Self {
        Self::from_bytes(value.0)
    }
}

impl From<Uuid> for sdbUuid {
    fn from(value: Uuid) -> Self {
        let tmp: uuid::Uuid = value.into();
        tmp.into()
    }
}
