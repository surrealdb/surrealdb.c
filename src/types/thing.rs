use std::ffi::{c_char, CString};

use surrealdb::sql;

use crate::{array::Array, object::Object};

#[repr(C)]
#[derive(Debug)]
pub struct Thing {
    table: *mut c_char,
    id: Id,
}

#[repr(C)]
#[derive(Debug)]
pub enum Id {
    IdNumber(i64),
    IdString(*mut c_char),
    // unnesessary Box, but breaks header gen
    IdArray(Box<Array>),
    IdObject(Object),
    // Generate(Gen),
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
