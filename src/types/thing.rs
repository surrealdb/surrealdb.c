use std::{
    ffi::{c_char, c_int, CStr, CString},
    ops::Bound,
};

use surrealdb::sql;

use crate::{array::Array, object::Object, string::string_t, utils::CStringExt2};

use super::uuid::Uuid;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Thing {
    table: string_t,
    id: Id,
}

impl Thing {
    #[export_name = "sr_thing_new_string"]
    pub fn new_string(table: *const c_char, id: *const c_char) -> Thing {
        let table = unsafe { CStr::from_ptr(table) }
            .to_string_lossy()
            .to_string_t();
        let id = unsafe { CStr::from_ptr(id) }
            .to_string_lossy()
            .to_string_t();
        Thing {
            table,
            id: Id::SR_ID_STRING(id),
        }
    }

    #[export_name = "sr_thing_new_int"]
    pub fn new_int(table: *const c_char, id: c_int) -> Thing {
        let table = unsafe { CStr::from_ptr(table) }
            .to_string_lossy()
            .to_string_t();
        Thing {
            table,
            id: Id::SR_ID_NUMBER(id as i64),
        }
    }
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
    SR_ID_UUID(Uuid),
    SR_ID_RANGE(IdRange),
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum BoundKind {
    SR_BOUND_INCLUDED,
    SR_BOUND_EXCLUDED,
    SR_BOUND_UNBOUNDED,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct IdRange {
    beg_kind: BoundKind,
    beg: Box<Id>,
    end_kind: BoundKind,
    end: Box<Id>,
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
            sql::Id::Uuid(id) => Id::SR_ID_UUID(id.into()),
            sql::Id::Range(_) => todo!(),
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
        let id = (&value.id).into();
        Self {
            table: string_t(str_ptr),
            id,
        }
    }
}

impl From<&sql::Id> for Id {
    fn from(value: &sql::Id) -> Self {
        match value {
            sql::Id::Number(i) => Id::SR_ID_NUMBER(*i),
            sql::Id::String(s) => Id::SR_ID_STRING(s.as_str().to_string_t()),
            sql::Id::Array(a) => Id::SR_ID_ARRAY(Box::new(a.into())),
            sql::Id::Object(o) => Id::SR_ID_OBJECT(o.into()),
            sql::Id::Generate(_) => todo!(),
            sql::Id::Uuid(id) => Id::SR_ID_UUID(id.clone().into()),
            sql::Id::Range(_) => todo!(),
            _ => todo!(),
        }
    }
}

impl From<&Id> for sql::Id {
    fn from(value: &Id) -> Self {
        match value {
            Id::SR_ID_NUMBER(i) => sql::Id::Number(*i),
            Id::SR_ID_STRING(s) => sql::Id::String(s.into()),
            Id::SR_ID_ARRAY(a) => sql::Id::Array(a.as_ref().into()),
            Id::SR_ID_OBJECT(o) => sql::Id::Object(o.into()),
            Id::SR_ID_UUID(i) => sql::Id::Uuid(i.into()),
            Id::SR_ID_RANGE(r) => todo!(),
        }
    }
}

// impl From<&sql::Range> for IdRange {
//     fn from(value: &sql::Range) -> Self {}
// }

fn from_sql_bound(value: &Bound<sql::Id>) -> (BoundKind, Option<Box<Id>>) {
    match value {
        Bound::Included(i) => (BoundKind::SR_BOUND_INCLUDED, Some(Box::new(i.into()))),
        Bound::Excluded(i) => (BoundKind::SR_BOUND_EXCLUDED, Some(Box::new(i.into()))),
        Bound::Unbounded => todo!(),
    }
}

impl From<Thing> for sql::Thing {
    fn from(value: Thing) -> Self {
        (&value).into()
    }
}

impl From<&Thing> for sql::Thing {
    fn from(value: &Thing) -> Self {
        let table = String::from(&value.table);
        let id: sql::Id = (&value.id).into();

        (table, id).into()
    }
}
