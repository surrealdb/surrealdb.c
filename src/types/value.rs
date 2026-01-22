use std::ffi::CStr;

use chrono::DateTime;
use surrealdb_core::sql;
use surrealdb_core::sql::Value as sdbValue;

pub use crate::{array::Array, number::Number, object::Object, geometry::sr_geometry};
use crate::{bytes::Bytes, string::string_t, thing::Thing, utils::CStringExt2, uuid::Uuid};

use super::duration::Duration;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub enum Value {
    #[default]
    SR_VALUE_NONE,
    SR_VALUE_NULL,
    SR_VALUE_BOOL(bool),
    SR_VALUE_NUMBER(Number),
    SR_VALUE_STRAND(string_t),
    SR_VALUE_DURATION(Duration),
    SR_VALUE_DATETIME(string_t),
    SR_VALUE_UUID(Uuid),
    SR_VALUE_ARRAY(Box<Array>),
    SR_VALUE_OBJECT(Object),
    SR_GEOMETRY_OBJECT(sr_geometry),
    SR_VALUE_BYTES(Bytes),
    SR_VALUE_THING(Thing),
    // TODO(Lance): Are computed objects needed?
}

impl From<sdbValue> for Value {
    fn from(value: sdbValue) -> Self {
        match value {
            sdbValue::None => Value::SR_VALUE_NONE,
            sdbValue::Null => Value::SR_VALUE_NULL,
            sdbValue::Bool(b) => Value::SR_VALUE_BOOL(b),
            sdbValue::Number(n) => match n {
                sql::Number::Int(i) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_INT(i)),
                sql::Number::Float(f) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(f)),
                sql::Number::Decimal(d) => Value::SR_VALUE_NUMBER(Number::from(d)),
                _ => {
                    // Handle any new Number variants by converting to string representation
                    Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(0.0))
                }
            },
            sdbValue::Strand(s) => Value::SR_VALUE_STRAND(s.0.to_string_t()),
            sdbValue::Duration(d) => Value::SR_VALUE_DURATION(d.into()),
            sdbValue::Datetime(dt) => Value::SR_VALUE_DATETIME(dt.to_rfc3339().to_string_t()),
            sdbValue::Uuid(u) => Value::SR_VALUE_UUID(u.into()),
            // unnecessary box see: https://github.com/mozilla/cbindgen/issues/981
            sdbValue::Array(a) => Value::SR_VALUE_ARRAY(Box::new(a.into())),
            sdbValue::Object(o) => Value::SR_VALUE_OBJECT(o.into()),
            sdbValue::Geometry(g) => Value::SR_GEOMETRY_OBJECT(sr_geometry::from(g)),
            sdbValue::Bytes(b) => Value::SR_VALUE_BYTES(b.into()),
            sdbValue::Thing(t) => Value::SR_VALUE_THING(t.into()),
            // Other variants are internal/computed and shouldn't appear in query results
            // If they do, we convert to None to avoid panics
            _ => Value::SR_VALUE_NONE,
        }
    }
}

impl From<&sdbValue> for Value {
    fn from(value: &sdbValue) -> Self {
        match value {
            sdbValue::None => Value::SR_VALUE_NONE,
            sdbValue::Null => Value::SR_VALUE_NULL,
            sdbValue::Bool(b) => Value::SR_VALUE_BOOL(*b),
            sdbValue::Number(n) => match n {
                sql::Number::Int(i) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_INT(*i)),
                sql::Number::Float(f) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(*f)),
                sql::Number::Decimal(d) => Value::SR_VALUE_NUMBER(Number::from(*d)),
                _ => {
                    // Handle any new Number variants
                    Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(0.0))
                }
            },
            sdbValue::Strand(s) => Value::SR_VALUE_STRAND(s.0.as_str().to_string_t()),
            sdbValue::Duration(d) => Value::SR_VALUE_DURATION(d.clone().into()),
            sdbValue::Datetime(dt) => Value::SR_VALUE_DATETIME(dt.to_rfc3339().to_string_t()),
            sdbValue::Uuid(u) => Value::SR_VALUE_UUID(u.clone().into()),
            // unnecessary box see: https://github.com/mozilla/cbindgen/issues/981
            sdbValue::Array(a) => Value::SR_VALUE_ARRAY(Box::new(a.into())),
            sdbValue::Object(o) => Value::SR_VALUE_OBJECT(o.into()),
            sdbValue::Geometry(g) => Value::SR_GEOMETRY_OBJECT(sr_geometry::from(g.clone())),
            sdbValue::Bytes(b) => Value::SR_VALUE_BYTES(b.clone().into()),
            sdbValue::Thing(t) => Value::SR_VALUE_THING(t.into()),
            // Other variants are internal/computed and shouldn't appear in query results
            _ => Value::SR_VALUE_NONE,
        }
    }
}
impl From<surrealdb::Value> for Value {
    fn from(value: surrealdb::Value) -> Self {
        let value: sdbValue = value.into_inner();
        Self::from(value)
    }
}
impl From<&surrealdb::Value> for Value {
    fn from(value: &surrealdb::Value) -> Self {
        // TODO: change to into_inner_ref when merged
        // SAFETY: surrealdb::Value is a transparent srapper
        let value: &sdbValue = unsafe { std::mem::transmute(value) };
        Self::from(value)
    }
}

impl From<Value> for sdbValue {
    fn from(value: Value) -> Self {
        match value {
            Value::SR_VALUE_NONE => sdbValue::None,
            Value::SR_VALUE_NULL => sdbValue::Null,
            Value::SR_VALUE_BOOL(b) => sdbValue::Bool(b),
            Value::SR_VALUE_NUMBER(n) => sdbValue::Number(n.into()),
            Value::SR_VALUE_STRAND(s) => sdbValue::Strand(String::from(s).into()),
            Value::SR_VALUE_DURATION(d) => sdbValue::Duration(d.into()),
            Value::SR_VALUE_DATETIME(d) => {
                let cstr = unsafe { CStr::from_ptr(d.0) };
                sdbValue::Datetime(
                    DateTime::parse_from_rfc3339(cstr.to_string_lossy().as_ref())
                        .unwrap_or_default()
                        .to_utc()
                        .into(),
                )
            }
            Value::SR_VALUE_UUID(u) => sdbValue::Uuid(u.into()),
            Value::SR_VALUE_ARRAY(a) => sdbValue::Array((*a).into()),
            Value::SR_VALUE_OBJECT(o) => sdbValue::Object(o.into()),
            Value::SR_GEOMETRY_OBJECT(g) => sdbValue::Geometry(g.into()),
            Value::SR_VALUE_BYTES(b) => sdbValue::Bytes(b.into()),
            Value::SR_VALUE_THING(t) => sdbValue::Thing(t.into()),
        }
    }
}

impl Value {
    #[export_name = "sr_value_print"]
    pub extern "C" fn print_value(val: &Value) {
        println!("{val:?}");
    }

    #[export_name = "sr_value_eq"]
    pub extern "C" fn value_eq(lhs: &Value, rhs: &Value) -> bool {
        lhs == rhs
    }

    /// Create a None value
    #[export_name = "sr_value_none"]
    pub extern "C" fn value_none() -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_NONE))
    }

    /// Create a Null value
    #[export_name = "sr_value_null"]
    pub extern "C" fn value_null() -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_NULL))
    }

    /// Create a Bool value
    #[export_name = "sr_value_bool"]
    pub extern "C" fn value_bool(val: bool) -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_BOOL(val)))
    }

    /// Create an Int value
    #[export_name = "sr_value_int"]
    pub extern "C" fn value_int(val: i64) -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_NUMBER(Number::SR_NUMBER_INT(val))))
    }

    /// Create a Float value
    #[export_name = "sr_value_float"]
    pub extern "C" fn value_float(val: f64) -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(val))))
    }

    /// Create a String value
    #[export_name = "sr_value_string"]
    pub extern "C" fn value_string(val: *const std::ffi::c_char) -> *mut Value {
        let s = unsafe { std::ffi::CStr::from_ptr(val) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_STRAND(s)))
    }

    /// Create an Object value from an existing object
    #[export_name = "sr_value_object"]
    pub extern "C" fn value_object(obj: *const Object) -> *mut Value {
        let obj = unsafe { &*obj }.clone();
        Box::into_raw(Box::new(Value::SR_VALUE_OBJECT(obj)))
    }

    /// Create a Duration value
    #[export_name = "sr_value_duration"]
    pub extern "C" fn value_duration(secs: u64, nanos: u32) -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_DURATION(Duration { secs, nanos })))
    }

    /// Create a Datetime value from RFC3339 string (e.g. "2024-01-15T10:30:00Z")
    #[export_name = "sr_value_datetime"]
    pub extern "C" fn value_datetime(val: *const std::ffi::c_char) -> *mut Value {
        let s = unsafe { std::ffi::CStr::from_ptr(val) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_DATETIME(s)))
    }

    /// Create a UUID value from 16 bytes
    #[export_name = "sr_value_uuid"]
    pub extern "C" fn value_uuid(bytes: *const u8) -> *mut Value {
        let bytes_slice = unsafe { std::slice::from_raw_parts(bytes, 16) };
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes_slice);
        Box::into_raw(Box::new(Value::SR_VALUE_UUID(Uuid(arr))))
    }

    /// Create an empty Array value
    #[export_name = "sr_value_array"]
    pub extern "C" fn value_array() -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_ARRAY(Box::new(Array {
            arr: std::ptr::null_mut(),
            len: 0,
        }))))
    }

    /// Create a Bytes value from raw data
    #[export_name = "sr_value_bytes"]
    pub extern "C" fn value_bytes(data: *const u8, len: std::ffi::c_int) -> *mut Value {
        let bytes = if data.is_null() || len <= 0 {
            Bytes {
                arr: std::ptr::null_mut(),
                len: 0,
            }
        } else {
            let slice = unsafe { std::slice::from_raw_parts(data, len as usize) };
            let vec = slice.to_vec();
            let boxed = vec.into_boxed_slice();
            let ptr = Box::into_raw(boxed) as *mut u8;
            Bytes { arr: ptr, len }
        };
        Box::into_raw(Box::new(Value::SR_VALUE_BYTES(bytes)))
    }

    /// Create a Thing value (record ID) from table name and string ID
    #[export_name = "sr_value_thing"]
    pub extern "C" fn value_thing(table: *const std::ffi::c_char, id: *const std::ffi::c_char) -> *mut Value {
        let table_str = unsafe { std::ffi::CStr::from_ptr(table) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        let id_str = unsafe { std::ffi::CStr::from_ptr(id) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_THING(Thing {
            table: table_str,
            id: crate::thing::Id::SR_ID_STRING(id_str),
        })))
    }

    /// Free a value created by sr_value_* functions
    #[export_name = "sr_value_free"]
    pub extern "C" fn value_free(val: *mut Value) {
        if !val.is_null() {
            let _ = unsafe { Box::from_raw(val) };
        }
    }

    /// Create a Point geometry value
    #[export_name = "sr_value_point"]
    pub extern "C" fn value_point(x: f64, y: f64) -> *mut Value {
        use crate::geometry::{sr_g_coord, sr_g_point, sr_geometry};
        let point = sr_g_point(sr_g_coord { x, y });
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_point(point))))
    }

    /// Create a LineString geometry value from an array of coordinates
    /// coords is a pointer to an array of sr_g_coord structures
    #[export_name = "sr_value_linestring"]
    pub extern "C" fn value_linestring(coords: *const crate::geometry::sr_g_coord, len: std::ffi::c_int) -> *mut Value {
        use crate::geometry::{sr_g_linestring, sr_geometry};
        use crate::array::MakeArray;
        
        if coords.is_null() || len <= 0 {
            // Return empty linestring
            let ls = sr_g_linestring(Vec::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_linestring(ls))));
        }
        
        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let vec: Vec<crate::geometry::sr_g_coord> = slice.to_vec();
        let ls = sr_g_linestring(vec.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_linestring(ls))))
    }

    /// Create a simple Polygon geometry value from exterior ring coordinates
    /// coords is a pointer to an array of sr_g_coord structures for the exterior ring
    #[export_name = "sr_value_polygon"]
    pub extern "C" fn value_polygon(coords: *const crate::geometry::sr_g_coord, len: std::ffi::c_int) -> *mut Value {
        use crate::geometry::{sr_g_linestring, sr_g_polygon, sr_geometry};
        use crate::array::MakeArray;
        
        if coords.is_null() || len <= 0 {
            // Return empty polygon
            let exterior = sr_g_linestring(Vec::new().make_array());
            let interiors: Vec<sr_g_linestring> = Vec::new();
            let poly = sr_g_polygon(exterior, interiors.make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_polygon(poly))));
        }
        
        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let vec: Vec<crate::geometry::sr_g_coord> = slice.to_vec();
        let exterior = sr_g_linestring(vec.make_array());
        let interiors: Vec<sr_g_linestring> = Vec::new();
        let poly = sr_g_polygon(exterior, interiors.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_polygon(poly))))
    }

    /// Create a MultiPoint geometry value from an array of points (x,y pairs)
    /// coords is a pointer to an array of sr_g_coord structures
    #[export_name = "sr_value_multipoint"]
    pub extern "C" fn value_multipoint(coords: *const crate::geometry::sr_g_coord, len: std::ffi::c_int) -> *mut Value {
        use crate::geometry::{sr_g_coord, sr_g_point, sr_g_multipoint, sr_geometry};
        use crate::array::MakeArray;
        
        if coords.is_null() || len <= 0 {
            // Return empty multipoint
            let mp = sr_g_multipoint(Vec::<sr_g_point>::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_multipoint(mp))));
        }
        
        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let points: Vec<sr_g_point> = slice.iter()
            .map(|c| sr_g_point(sr_g_coord { x: c.x, y: c.y }))
            .collect();
        let mp = sr_g_multipoint(points.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_multipoint(mp))))
    }
}
