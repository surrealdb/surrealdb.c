use std::ffi::CString;

use surrealdb_core::sql;
use surrealdb_core::sql::Value as sdbValue;

#[repr(C)]
#[derive(Default)]
pub enum Value {
    #[default]
    None,
    Null,
    Bool(bool),
    Number(Number),
    Strand(CString),
    // Duration(Duration),
    // Datetime(Datetime),
    // Uuid(Uuid),
    // Array(Array),
    // Object(Object),
    // Geometry(Geometry),
    // Bytes(Bytes),
    // Thing(Thing),
    // Param(Param),
    // Idiom(Idiom),
    // Table(Table),
    // Mock(Mock),
    // Regex(Regex),
    // Cast(Box<Cast>),
    // Block(Box<Block>),
    // Range(Box<Range>),
    // Edges(Box<Edges>),
    // Future(Box<Future>),
    // Constant(Constant),
    // Function(Box<Function>),
    // Subquery(Box<Subquery>),
    // Expression(Box<Expression>),
    // Query(Query),
    // Model(Box<Model>),
}

impl From<sdbValue> for Value {
    fn from(value: sdbValue) -> Self {
        match value {
            sdbValue::None => Value::None,
            sdbValue::Null => Value::Null,
            sdbValue::Bool(b) => Value::Bool(b),
            sdbValue::Number(n) => match n {
                sql::Number::Int(i) => Value::Number(Number::Int(i)),
                sql::Number::Float(f) => Value::Number(Number::Float(f)),
                sql::Number::Decimal(_) => todo!(),
                _ => todo!(),
            },
            sdbValue::Strand(_) => todo!(),
            sdbValue::Duration(_) => todo!(),
            sdbValue::Datetime(_) => todo!(),
            sdbValue::Uuid(_) => todo!(),
            sdbValue::Array(_) => todo!(),
            sdbValue::Object(_) => todo!(),
            sdbValue::Geometry(_) => todo!(),
            sdbValue::Bytes(_) => todo!(),
            sdbValue::Thing(_) => todo!(),
            sdbValue::Param(_) => todo!(),
            sdbValue::Idiom(_) => todo!(),
            sdbValue::Table(_) => todo!(),
            sdbValue::Mock(_) => todo!(),
            sdbValue::Regex(_) => todo!(),
            sdbValue::Cast(_) => todo!(),
            sdbValue::Block(_) => todo!(),
            sdbValue::Range(_) => todo!(),
            sdbValue::Edges(_) => todo!(),
            sdbValue::Future(_) => todo!(),
            sdbValue::Constant(_) => todo!(),
            sdbValue::Function(_) => todo!(),
            sdbValue::Subquery(_) => todo!(),
            sdbValue::Expression(_) => todo!(),
            sdbValue::Query(_) => todo!(),
            sdbValue::Model(_) => todo!(),
            _ => todo!(),
        }
    }
}

#[repr(C)]
pub enum Number {
    Int(i64),
    Float(f64),
}
