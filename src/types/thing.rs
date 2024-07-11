use std::{ffi::CString, fmt::Debug};

use surrealdb::sql;

use crate::{array::Array, object::Object, string::string_t, utils::CStringExt2};

#[repr(C)]
#[derive(Debug)]
pub struct Thing {
    table: string_t,
    id: Id,
}

#[repr(C)]
#[derive(Debug)]
pub enum Id {
    IdNumber(i64),
    IdString(string_t),
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
            sql::Id::String(s) => Id::IdString(s.to_string_t()),
            sql::Id::Array(a) => Id::IdArray(Box::new(a.into())),
            sql::Id::Object(o) => Id::IdObject(o.into()),
            sql::Id::Generate(_) => todo!(),
            _ => todo!(),
        };
        Self {
            table: string_t(str_ptr),
            id,
        }
    }
}
