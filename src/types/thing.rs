use std::{
    ffi::{c_char, CString},
    fmt::Debug,
};

use surrealdb::sql;

use crate::{array::Array, object::Object, ptr_to_str};

#[repr(C)]
pub struct Thing {
    table: *mut c_char,
    id: Id,
}

impl Debug for Thing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Thing")
            .field("table", &ptr_to_str(self.table))
            .field("id", &self.id)
            .finish()
    }
}

#[repr(C)]
pub enum Id {
    IdNumber(i64),
    IdString(*mut c_char),
    // unnesessary Box, but breaks header gen
    IdArray(Box<Array>),
    IdObject(Object),
    // Generate(Gen),
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IdNumber(arg0) => f.debug_tuple("IdNumber").field(arg0).finish(),
            Self::IdString(arg0) => f.debug_tuple("IdString").field(&ptr_to_str(*arg0)).finish(),
            Self::IdArray(arg0) => f.debug_tuple("IdArray").field(arg0).finish(),
            Self::IdObject(arg0) => f.debug_tuple("IdObject").field(arg0).finish(),
        }
    }
}

impl From<sql::Thing> for Thing {
    fn from(value: sql::Thing) -> Self {
        let str_ptr = CString::new(value.tb).unwrap().into_raw();
        let id = match value.id {
            sql::Id::Number(i) => Id::IdNumber(i),
            sql::Id::String(s) => Id::IdString(CString::new(s).unwrap().into_raw()),
            sql::Id::Array(a) => Id::IdArray(Box::new(a.into())),
            sql::Id::Object(o) => Id::IdObject(o.into()),
            sql::Id::Generate(_) => todo!(),
            _ => todo!(),
        };
        Self { table: str_ptr, id }
    }
}
