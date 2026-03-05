use std::{ffi::CString, fmt::Debug};

use surrealdb::types::{RecordId, RecordIdKey};

use crate::{array::Array, object::Object, string::string_t, utils::CStringExt2};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Thing {
    pub table: string_t,
    pub id: Id,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum Id {
    SR_ID_NUMBER(i64),
    SR_ID_STRING(string_t),
    SR_ID_ARRAY(Box<Array>),
    SR_ID_OBJECT(Object),
}

impl From<RecordId> for Thing {
    fn from(value: RecordId) -> Self {
        let table_str = value.table.to_string();
        let str_ptr = CString::new(table_str).unwrap_or_else(|_| CString::new("").unwrap()).into_raw();
        let id = match value.key {
            RecordIdKey::Number(i) => Id::SR_ID_NUMBER(i),
            RecordIdKey::String(s) => Id::SR_ID_STRING(s.to_string_t()),
            RecordIdKey::Array(a) => Id::SR_ID_ARRAY(Box::new(Array::from(a))),
            RecordIdKey::Object(o) => Id::SR_ID_OBJECT(Object::from(o)),
            other => Id::SR_ID_STRING(format!("{:?}", other).to_string_t()),
        };
        Self {
            table: string_t(str_ptr),
            id,
        }
    }
}

impl From<&RecordId> for Thing {
    fn from(value: &RecordId) -> Self {
        Self::from(value.clone())
    }
}

impl From<Thing> for RecordId {
    fn from(value: Thing) -> Self {
        let table = String::from(value.table);
        let key = match value.id {
            Id::SR_ID_NUMBER(i) => RecordIdKey::Number(i),
            Id::SR_ID_STRING(s) => RecordIdKey::String(s.into()),
            Id::SR_ID_ARRAY(a) => RecordIdKey::Array((*a).into()),
            Id::SR_ID_OBJECT(o) => RecordIdKey::Object(o.into()),
        };
        RecordId {
            table: table.into(),
            key,
        }
    }
}
