use std::{ffi::CString, fmt::Debug};

use surrealdb::sql;

use crate::{array::Array, object::Object, string::string_t, utils::CStringExt2};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Thing {
    table: string_t,
    id: Id,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum Id {
    SR_ID_NUMBER(i64),
    SR_ID_STRING(string_t),
    // unnesessary Box, but breaks header gen
    SR_ID_ARRAY(Box<Array>),
    SR_ID_OBJECT(Object),
    // Generate(Gen),
}

impl From<sql::Thing> for Thing {
    fn from(value: sql::Thing) -> Self {
        let str_ptr = CString::new(value.tb).unwrap().into_raw();
        let id = match value.id {
            sql::Id::Number(i) => Id::SR_ID_NUMBER(i),
            sql::Id::String(s) => Id::SR_ID_STRING(s.to_string_t()),
            sql::Id::Array(a) => Id::SR_ID_ARRAY(Box::new(a.into())),
            sql::Id::Object(o) => Id::SR_ID_OBJECT(o.into()),
            sql::Id::Generate(_) => todo!(),
            _ => todo!(),
        };
        Self {
            table: string_t(str_ptr),
            id,
        }
    }
}

impl From<&sql::Thing> for Thing {
    fn from(value: &sql::Thing) -> Self {
        let str_ptr = CString::new(value.tb.clone()).unwrap().into_raw();
        let id = match &value.id {
            sql::Id::Number(i) => Id::SR_ID_NUMBER(*i),
            sql::Id::String(s) => Id::SR_ID_STRING(s.as_str().to_string_t()),
            sql::Id::Array(a) => Id::SR_ID_ARRAY(Box::new(a.into())),
            sql::Id::Object(o) => Id::SR_ID_OBJECT(o.into()),
            sql::Id::Generate(_) => todo!(),
            _ => todo!(),
        };
        Self {
            table: string_t(str_ptr),
            id,
        }
    }
}

impl From<Thing> for sql::Thing {
    fn from(value: Thing) -> Self {
        let table = String::from(value.table);
        let id = match value.id {
            Id::SR_ID_NUMBER(i) => sql::Id::Number(i),
            Id::SR_ID_STRING(s) => sql::Id::String(s.into()),
            Id::SR_ID_ARRAY(a) => sql::Id::Array((*a).into()),
            Id::SR_ID_OBJECT(o) => sql::Id::Object(o.into()),
        };

        (table, id).into()
    }
}
