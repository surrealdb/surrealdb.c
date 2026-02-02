use std::ffi::CStr;

use chrono::DateTime;
use surrealdb_core::sql;
use surrealdb_core::sql::Value as sdbValue;

use super::duration::Duration;
pub use crate::array::Array;
use crate::bytes::Bytes;
pub use crate::geometry::sr_geometry;
pub use crate::number::Number;
pub use crate::object::Object;
use crate::string::string_t;
use crate::thing::Thing;
use crate::utils::CStringExt2;
use crate::uuid::Uuid;

/// Represents a SurrealDB value
///
/// This enum wraps all possible value types that can be returned from SurrealDB queries
/// or used as input parameters. Each variant corresponds to a SurrealDB data type.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub enum Value {
    /// No value (absence of data)
    #[default]
    SR_VALUE_NONE,
    /// Explicit null value
    SR_VALUE_NULL,
    /// Boolean value (true/false)
    SR_VALUE_BOOL(bool),
    /// Numeric value (integer, float, or decimal)
    SR_VALUE_NUMBER(Number),
    /// String value
    SR_VALUE_STRAND(string_t),
    /// Duration value
    SR_VALUE_DURATION(Duration),
    /// DateTime value in RFC3339 format
    SR_VALUE_DATETIME(string_t),
    /// UUID value
    SR_VALUE_UUID(Uuid),
    /// Array of values
    SR_VALUE_ARRAY(Box<Array>),
    /// Object (key-value map)
    SR_VALUE_OBJECT(Object),
    /// Geometry object (points, lines, polygons, etc.)
    SR_GEOMETRY_OBJECT(sr_geometry),
    /// Raw bytes
    SR_VALUE_BYTES(Bytes),
    /// Record ID (thing)
    SR_VALUE_THING(Thing),
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
            sdbValue::Duration(d) => Value::SR_VALUE_DURATION((*d).into()),
            sdbValue::Datetime(dt) => Value::SR_VALUE_DATETIME(dt.to_rfc3339().to_string_t()),
            sdbValue::Uuid(u) => Value::SR_VALUE_UUID((*u).into()),
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
        // Clone the value to safely convert without relying on internal layout assumptions
        Self::from(value.clone())
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
    /// Print a value to stdout for debugging
    ///
    /// Outputs the debug representation of the value to standard output.
    #[export_name = "sr_value_print"]
    pub extern "C" fn print_value(val: &Value) {
        println!("{val:?}");
    }

    /// Compare two values for equality
    ///
    /// Returns true if both values are equal, false otherwise.
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
    ///
    /// # Safety
    ///
    /// - `val` must be a valid pointer to a null-terminated UTF-8 string
    #[export_name = "sr_value_string"]
    pub unsafe extern "C" fn value_string(val: *const std::ffi::c_char) -> *mut Value {
        let s =
            unsafe { std::ffi::CStr::from_ptr(val) }.to_string_lossy().to_string().to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_STRAND(s)))
    }

    /// Create an Object value from an existing object
    ///
    /// # Safety
    ///
    /// - `obj` must be a valid pointer to an Object
    #[export_name = "sr_value_object"]
    pub unsafe extern "C" fn value_object(obj: *const Object) -> *mut Value {
        let obj = unsafe { &*obj }.clone();
        Box::into_raw(Box::new(Value::SR_VALUE_OBJECT(obj)))
    }

    /// Create a Duration value
    #[export_name = "sr_value_duration"]
    pub extern "C" fn value_duration(secs: u64, nanos: u32) -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_DURATION(Duration {
            secs,
            nanos,
        })))
    }

    /// Create a Datetime value from RFC3339 string (e.g. "2024-01-15T10:30:00Z")
    ///
    /// # Safety
    ///
    /// - `val` must be a valid pointer to a null-terminated UTF-8 string
    #[export_name = "sr_value_datetime"]
    pub unsafe extern "C" fn value_datetime(val: *const std::ffi::c_char) -> *mut Value {
        let s =
            unsafe { std::ffi::CStr::from_ptr(val) }.to_string_lossy().to_string().to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_DATETIME(s)))
    }

    /// Create a UUID value from 16 bytes
    ///
    /// # Safety
    ///
    /// - `bytes` must be a valid pointer to 16 bytes
    #[export_name = "sr_value_uuid"]
    pub unsafe extern "C" fn value_uuid(bytes: *const u8) -> *mut Value {
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
    ///
    /// # Safety
    ///
    /// - `data` must be a valid pointer to raw data
    /// - `len` must be the length of the data
    #[export_name = "sr_value_bytes"]
    pub unsafe extern "C" fn value_bytes(data: *const u8, len: std::ffi::c_int) -> *mut Value {
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
            Bytes {
                arr: ptr,
                len,
            }
        };
        Box::into_raw(Box::new(Value::SR_VALUE_BYTES(bytes)))
    }

    /// Create a Thing value (record ID) from table name and string ID
    ///
    /// # Safety
    ///
    /// - `table` must be a valid pointer to a null-terminated UTF-8 string
    /// - `id` must be a valid pointer to a null-terminated UTF-8 string
    #[export_name = "sr_value_thing"]
    pub unsafe extern "C" fn value_thing(
        table: *const std::ffi::c_char,
        id: *const std::ffi::c_char,
    ) -> *mut Value {
        let table_str =
            unsafe { std::ffi::CStr::from_ptr(table) }.to_string_lossy().to_string().to_string_t();
        let id_str =
            unsafe { std::ffi::CStr::from_ptr(id) }.to_string_lossy().to_string().to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_THING(Thing {
            table: table_str,
            id: crate::thing::Id::SR_ID_STRING(id_str),
        })))
    }

    /// Free a value created by sr_value_* functions
    ///
    /// # Safety
    ///
    /// - `val` must be a valid pointer to a Value
    #[export_name = "sr_value_free"]
    pub unsafe extern "C" fn value_free(val: *mut Value) {
        if !val.is_null() {
            let _ = unsafe { Box::from_raw(val) };
        }
    }

    /// Create a Point geometry value
    #[export_name = "sr_value_point"]
    pub extern "C" fn value_point(x: f64, y: f64) -> *mut Value {
        use crate::geometry::{sr_g_coord, sr_g_point, sr_geometry};
        let point = sr_g_point(sr_g_coord {
            x,
            y,
        });
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_point(point))))
    }

    /// Create a LineString geometry value from an array of coordinates
    /// coords is a pointer to an array of sr_g_coord structures
    ///
    /// # Safety
    ///
    /// - `coords` must be a valid pointer to an array of sr_g_coord structures
    /// - `len` must be the length of the array
    #[export_name = "sr_value_linestring"]
    pub unsafe extern "C" fn value_linestring(
        coords: *const crate::geometry::sr_g_coord,
        len: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_linestring, sr_geometry};

        if coords.is_null() || len <= 0 {
            // Return empty linestring
            let ls = sr_g_linestring(Vec::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
                sr_geometry::sr_g_linestring(ls),
            )));
        }

        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let vec: Vec<crate::geometry::sr_g_coord> = slice.to_vec();
        let ls = sr_g_linestring(vec.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_linestring(ls))))
    }

    /// Create a simple Polygon geometry value from exterior ring coordinates
    /// coords is a pointer to an array of sr_g_coord structures for the exterior ring
    ///
    /// # Safety
    ///
    /// - `coords` must be a valid pointer to an array of sr_g_coord structures
    /// - `len` must be the length of the array
    #[export_name = "sr_value_polygon"]
    pub unsafe extern "C" fn value_polygon(
        coords: *const crate::geometry::sr_g_coord,
        len: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_linestring, sr_g_polygon, sr_geometry};

        if coords.is_null() || len <= 0 {
            // Return empty polygon
            let exterior = sr_g_linestring(Vec::new().make_array());
            let interiors: Vec<sr_g_linestring> = Vec::new();
            let poly = sr_g_polygon(exterior, interiors.make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_polygon(
                poly,
            ))));
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
    ///
    /// # Safety
    ///
    /// - `coords` must be a valid pointer to an array of sr_g_coord structures
    /// - `len` must be the length of the array
    #[export_name = "sr_value_multipoint"]
    pub unsafe extern "C" fn value_multipoint(
        coords: *const crate::geometry::sr_g_coord,
        len: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_coord, sr_g_multipoint, sr_g_point, sr_geometry};

        if coords.is_null() || len <= 0 {
            // Return empty multipoint
            let mp = sr_g_multipoint(Vec::<sr_g_point>::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
                sr_geometry::sr_g_multipoint(mp),
            )));
        }

        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let points: Vec<sr_g_point> = slice
            .iter()
            .map(|c| {
                sr_g_point(sr_g_coord {
                    x: c.x,
                    y: c.y,
                })
            })
            .collect();
        let mp = sr_g_multipoint(points.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_multipoint(mp))))
    }

    /// Create a MultiLineString geometry value
    /// linestrings is an array of pointers to coordinate arrays
    /// lens is an array of lengths for each linestring
    /// count is the number of linestrings
    ///
    /// # Safety
    ///
    /// - `linestrings` must be a valid pointer to an array of pointers to coordinate arrays
    /// - `lens` must be a valid pointer to an array of lengths for each linestring
    /// - `count` must be the number of linestrings
    #[export_name = "sr_value_multilinestring"]
    pub unsafe extern "C" fn value_multilinestring(
        linestrings: *const *const crate::geometry::sr_g_coord,
        lens: *const std::ffi::c_int,
        count: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_linestring, sr_g_multilinestring, sr_geometry};

        if linestrings.is_null() || lens.is_null() || count <= 0 {
            let mls = sr_g_multilinestring(Vec::<sr_g_linestring>::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
                sr_geometry::sr_g_multiline(mls),
            )));
        }

        let linestring_ptrs = unsafe { std::slice::from_raw_parts(linestrings, count as usize) };
        let lengths = unsafe { std::slice::from_raw_parts(lens, count as usize) };

        let lines: Vec<sr_g_linestring> = linestring_ptrs
            .iter()
            .zip(lengths.iter())
            .map(|(&coords_ptr, &len)| {
                if coords_ptr.is_null() || len <= 0 {
                    sr_g_linestring(Vec::new().make_array())
                } else {
                    let slice = unsafe { std::slice::from_raw_parts(coords_ptr, len as usize) };
                    sr_g_linestring(slice.to_vec().make_array())
                }
            })
            .collect();

        let mls = sr_g_multilinestring(lines.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_multiline(mls))))
    }

    /// Create a MultiPolygon geometry value
    /// polygons is an array of pointers to coordinate arrays (exterior rings only)
    /// lens is an array of lengths for each polygon's exterior ring
    /// count is the number of polygons
    ///
    /// # Safety
    ///
    /// - `polygons` must be a valid pointer to an array of pointers to coordinate arrays
    /// - `lens` must be a valid pointer to an array of lengths for each polygon's exterior ring
    /// - `count` must be the number of polygons
    #[export_name = "sr_value_multipolygon"]
    pub unsafe extern "C" fn value_multipolygon(
        polygons: *const *const crate::geometry::sr_g_coord,
        lens: *const std::ffi::c_int,
        count: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_linestring, sr_g_multipolygon, sr_g_polygon, sr_geometry};

        if polygons.is_null() || lens.is_null() || count <= 0 {
            let mpoly = sr_g_multipolygon(Vec::<sr_g_polygon>::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
                sr_geometry::sr_g_multipolygon(mpoly),
            )));
        }

        let polygon_ptrs = unsafe { std::slice::from_raw_parts(polygons, count as usize) };
        let lengths = unsafe { std::slice::from_raw_parts(lens, count as usize) };

        let polys: Vec<sr_g_polygon> = polygon_ptrs
            .iter()
            .zip(lengths.iter())
            .map(|(&coords_ptr, &len)| {
                let exterior = if coords_ptr.is_null() || len <= 0 {
                    sr_g_linestring(Vec::new().make_array())
                } else {
                    let slice = unsafe { std::slice::from_raw_parts(coords_ptr, len as usize) };
                    sr_g_linestring(slice.to_vec().make_array())
                };
                let interiors: Vec<sr_g_linestring> = Vec::new();
                sr_g_polygon(exterior, interiors.make_array())
            })
            .collect();

        let mpoly = sr_g_multipolygon(polys.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::sr_g_multipolygon(mpoly))))
    }

    /// Create a Decimal value from string representation
    ///
    /// # Safety
    ///
    /// - `val` must be a valid pointer to a null-terminated UTF-8 string
    #[export_name = "sr_value_decimal"]
    pub unsafe extern "C" fn value_decimal(val: *const std::ffi::c_char) -> *mut Value {
        let s =
            unsafe { std::ffi::CStr::from_ptr(val) }.to_string_lossy().to_string().to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_NUMBER(Number::SR_NUMBER_DECIMAL(s))))
    }
}
