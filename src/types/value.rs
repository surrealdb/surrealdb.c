use std::ffi::CStr;

use chrono::DateTime;
use surrealdb::types::{
    Value as sdbValue, Number as sdbNumber, Set as sdbSet, Table as sdbTable,
};

pub use crate::{array::Array, number::Number, object::Object, geometry::sr_geometry};
pub use crate::{file::File, range::Range};
pub use crate::range::Bound;
use crate::array::ArrayGen;
use crate::{bytes::Bytes, string::string_t, thing::Thing, utils::CStringExt2, uuid::Uuid};

use super::duration::Duration;

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
    /// Table name
    SR_VALUE_TABLE(string_t),
    /// Reference to a file in a bucket
    SR_VALUE_FILE(File),
    /// Range between two bounds
    SR_VALUE_RANGE(Box<Range>),
    /// Regular expression, as its source pattern
    SR_VALUE_REGEX(string_t),
    /// Set of values. Distinct from SR_VALUE_ARRAY: elements are unique.
    SR_VALUE_SET(Box<Array>),
}

impl From<sdbValue> for Value {
    fn from(value: sdbValue) -> Self {
        match value {
            sdbValue::None => Value::SR_VALUE_NONE,
            sdbValue::Null => Value::SR_VALUE_NULL,
            sdbValue::Bool(b) => Value::SR_VALUE_BOOL(b),
            sdbValue::Number(n) => match n {
                sdbNumber::Int(i) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_INT(i)),
                sdbNumber::Float(f) => Value::SR_VALUE_NUMBER(Number::SR_NUMBER_FLOAT(f)),
                sdbNumber::Decimal(d) => Value::SR_VALUE_NUMBER(Number::from(d)),
            },
            sdbValue::String(s) => Value::SR_VALUE_STRAND(s.to_string_t()),
            sdbValue::Duration(d) => Value::SR_VALUE_DURATION(Duration::from(std::time::Duration::from(d))),
            sdbValue::Datetime(dt) => Value::SR_VALUE_DATETIME(dt.to_string().to_string_t()),
            sdbValue::Uuid(u) => Value::SR_VALUE_UUID(Uuid(u.into_bytes())),
            sdbValue::Array(a) => Value::SR_VALUE_ARRAY(Box::new(a.into())),
            sdbValue::Object(o) => Value::SR_VALUE_OBJECT(o.into()),
            sdbValue::Geometry(g) => Value::SR_GEOMETRY_OBJECT(sr_geometry::from(g)),
            sdbValue::Bytes(b) => Value::SR_VALUE_BYTES(Bytes::from(b)),
            sdbValue::RecordId(r) => Value::SR_VALUE_THING(Thing::from(r)),
            sdbValue::Table(t) => Value::SR_VALUE_TABLE(t.into_string().to_string_t()),
            sdbValue::File(f) => Value::SR_VALUE_FILE(File::from(f)),
            sdbValue::Range(r) => Value::SR_VALUE_RANGE(Box::new(Range::from(*r))),
            sdbValue::Regex(r) => Value::SR_VALUE_REGEX(r.to_string().to_string_t()),
            sdbValue::Set(s) => Value::SR_VALUE_SET(Box::new(Array::from(
                Vec::<sdbValue>::from(s)
                    .into_iter()
                    .map(Value::from)
                    .collect::<Vec<Value>>(),
            ))),
        }
    }
}

impl From<&sdbValue> for Value {
    fn from(value: &sdbValue) -> Self {
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
            Value::SR_VALUE_STRAND(s) => sdbValue::String(String::from(s)),
            Value::SR_VALUE_DURATION(d) => {
                let std_dur = std::time::Duration::new(d.secs, d.nanos);
                sdbValue::Duration(std_dur.into())
            }
            Value::SR_VALUE_DATETIME(d) => {
                let cstr = unsafe { CStr::from_ptr(d.0) };
                let parsed = DateTime::parse_from_rfc3339(cstr.to_string_lossy().as_ref())
                    .unwrap_or_default()
                    .to_utc();
                sdbValue::Datetime(parsed.into())
            }
            Value::SR_VALUE_UUID(u) => {
                let uuid_val: uuid::Uuid = u.into();
                sdbValue::Uuid(uuid_val.into())
            }
            Value::SR_VALUE_ARRAY(a) => sdbValue::Array((*a).into()),
            Value::SR_VALUE_OBJECT(o) => sdbValue::Object(o.into()),
            Value::SR_GEOMETRY_OBJECT(g) => sdbValue::Geometry(g.into()),
            Value::SR_VALUE_BYTES(b) => sdbValue::Bytes(b.into()),
            Value::SR_VALUE_THING(t) => sdbValue::RecordId(t.into()),
            Value::SR_VALUE_TABLE(t) => sdbValue::Table(sdbTable::new(String::from(t))),
            Value::SR_VALUE_FILE(f) => sdbValue::File(f.into()),
            Value::SR_VALUE_RANGE(r) => sdbValue::Range(Box::new((*r).into())),
            Value::SR_VALUE_REGEX(r) => match String::from(r).parse() {
                Ok(regex) => sdbValue::Regex(regex),
                Err(_) => sdbValue::None,
            },
            Value::SR_VALUE_SET(a) => sdbValue::Set(sdbSet::from(
                ArrayGen::<Value>::from(*a)
                    .into_vec()
                    .into_iter()
                    .map(sdbValue::from)
                    .collect::<Vec<sdbValue>>(),
            )),
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
    /// Create a table-name value
    ///
    /// # Safety
    ///
    /// - `name` must be a valid null-terminated string
    ///
    /// Free with sr_value_free
    #[export_name = "sr_value_table"]
    pub extern "C" fn value_table(name: *const std::ffi::c_char) -> *mut Value {
        if name.is_null() {
            return Box::into_raw(Box::new(Value::SR_VALUE_NONE));
        }
        let s = unsafe { std::ffi::CStr::from_ptr(name) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_TABLE(s)))
    }

    /// Create a file reference value
    ///
    /// # Safety
    ///
    /// - `bucket` and `key` must be valid null-terminated strings
    ///
    /// Free with sr_value_free
    #[export_name = "sr_value_file"]
    pub extern "C" fn value_file(
        bucket: *const std::ffi::c_char,
        key: *const std::ffi::c_char,
    ) -> *mut Value {
        if bucket.is_null() || key.is_null() {
            return Box::into_raw(Box::new(Value::SR_VALUE_NONE));
        }
        let bucket = unsafe { std::ffi::CStr::from_ptr(bucket) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        let key = unsafe { std::ffi::CStr::from_ptr(key) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_FILE(File { bucket, key })))
    }

    /// Create a regular-expression value from its source pattern
    ///
    /// The pattern is not validated here; an invalid pattern becomes SR_VALUE_NONE
    /// when the value is sent to the database.
    ///
    /// # Safety
    ///
    /// - `pattern` must be a valid null-terminated string
    ///
    /// Free with sr_value_free
    #[export_name = "sr_value_regex"]
    pub extern "C" fn value_regex(pattern: *const std::ffi::c_char) -> *mut Value {
        if pattern.is_null() {
            return Box::into_raw(Box::new(Value::SR_VALUE_NONE));
        }
        let s = unsafe { std::ffi::CStr::from_ptr(pattern) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_REGEX(s)))
    }

    /// Create a Set value from an array
    ///
    /// A set holds unique values; duplicates are discarded when it reaches the
    /// database. Build the array with `sr_array_from_values` or
    /// `sr_array_push`, then pass it here. A null pointer yields an empty set.
    ///
    /// The array is copied, so the caller keeps ownership of the one it passed
    /// in and must still release it with `sr_array_free`.
    ///
    /// Free with sr_value_free
    ///
    /// # Safety
    ///
    /// - `arr` must be null, or point to a valid Array
    #[export_name = "sr_value_set"]
    pub extern "C" fn value_set(arr: *const Array) -> *mut Value {
        let inner = if arr.is_null() {
            Array::empty()
        } else {
            unsafe { &*arr }.clone()
        };
        Box::into_raw(Box::new(Value::SR_VALUE_SET(Box::new(inner))))
    }

    /// An open bound, for a range that is unbounded at one end
    #[export_name = "sr_bound_unbounded"]
    pub extern "C" fn bound_unbounded() -> Bound {
        Bound::SR_BOUND_UNBOUNDED
    }

    /// A bound that includes `val`
    ///
    /// # Safety
    ///
    /// - `val` must be a valid pointer from a sr_value_* constructor. Ownership
    ///   transfers to the bound; do not free it separately.
    #[export_name = "sr_bound_included"]
    pub extern "C" fn bound_included(val: *mut Value) -> Bound {
        if val.is_null() {
            return Bound::SR_BOUND_UNBOUNDED;
        }
        Bound::SR_BOUND_INCLUDED(unsafe { Box::from_raw(val) })
    }

    /// A bound that stops before `val`
    ///
    /// # Safety
    ///
    /// - `val` must be a valid pointer from a sr_value_* constructor. Ownership
    ///   transfers to the bound; do not free it separately.
    #[export_name = "sr_bound_excluded"]
    pub extern "C" fn bound_excluded(val: *mut Value) -> Bound {
        if val.is_null() {
            return Bound::SR_BOUND_UNBOUNDED;
        }
        Bound::SR_BOUND_EXCLUDED(unsafe { Box::from_raw(val) })
    }

    /// Create a range value between two bounds
    ///
    /// Both bounds are consumed.
    ///
    /// Free with sr_value_free
    #[export_name = "sr_value_range"]
    pub extern "C" fn value_range(start: Bound, end: Bound) -> *mut Value {
        Box::into_raw(Box::new(Value::SR_VALUE_RANGE(Box::new(Range {
            start,
            end,
        }))))
    }

    /// Create an Array value from an array
    ///
    /// Build the array with `sr_array_from_values` or `sr_array_push`, then
    /// pass it here. A null pointer yields an empty array.
    ///
    /// The array is copied, so the caller keeps ownership of the one it passed
    /// in and must still release it with `sr_array_free`.
    ///
    /// Free with sr_value_free
    ///
    /// # Safety
    ///
    /// - `arr` must be null, or point to a valid Array
    #[export_name = "sr_value_array"]
    pub extern "C" fn value_array(arr: *const Array) -> *mut Value {
        let inner = if arr.is_null() {
            Array::empty()
        } else {
            unsafe { &*arr }.clone()
        };
        Box::into_raw(Box::new(Value::SR_VALUE_ARRAY(Box::new(inner))))
    }

    /// Create a Polygon geometry value from a list of rings
    ///
    /// `rings[0]` is the exterior ring; every ring after it is a hole. This is
    /// the shape of GeoJSON's `coordinates` array, so a caller that already has
    /// GeoJSON can pass it through unchanged.
    ///
    /// `lens` gives the coordinate count of each ring. The coordinates are
    /// copied, so the caller keeps ownership of the blocks it passed in.
    ///
    /// `sr_value_polygon` is the single-ring shorthand and cannot express a
    /// hole; use this whenever the polygon has one. A null pointer or a
    /// non-positive `ring_count` yields an empty polygon.
    ///
    /// Free with sr_value_free
    ///
    /// # Safety
    ///
    /// - `rings` must be null, or point to at least `ring_count` pointers
    /// - `lens` must be null, or point to at least `ring_count` lengths
    /// - each non-null `rings[i]` must point to at least `lens[i]` coordinates
    #[export_name = "sr_value_polygon_rings"]
    pub extern "C" fn value_polygon_rings(
        rings: *const *const crate::geometry::sr_g_coord,
        lens: *const std::ffi::c_int,
        ring_count: std::ffi::c_int,
    ) -> *mut Value {
        use crate::geometry::sr_geometry;

        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
            sr_geometry::SR_GEOMETRY_POLYGON(Self::build_polygon(rings, lens, ring_count)),
        )))
    }

    /// Shared by `sr_value_polygon_rings` and `sr_value_multipolygon_from`.
    fn build_polygon(
        rings: *const *const crate::geometry::sr_g_coord,
        lens: *const std::ffi::c_int,
        ring_count: std::ffi::c_int,
    ) -> crate::geometry::sr_g_polygon {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_linestring, sr_g_polygon};

        let empty = || sr_g_linestring(Vec::new().make_array());

        if rings.is_null() || lens.is_null() || ring_count <= 0 {
            return sr_g_polygon(empty(), Vec::<sr_g_linestring>::new().make_array());
        }

        let ring_ptrs = unsafe { std::slice::from_raw_parts(rings, ring_count as usize) };
        let lengths = unsafe { std::slice::from_raw_parts(lens, ring_count as usize) };

        let ring_at = |i: usize| -> sr_g_linestring {
            let (ptr, len) = (ring_ptrs[i], lengths[i]);
            if ptr.is_null() || len <= 0 {
                empty()
            } else {
                let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
                sr_g_linestring(slice.to_vec().make_array())
            }
        };

        let exterior = ring_at(0);
        let interiors: Vec<sr_g_linestring> = (1..ring_ptrs.len()).map(ring_at).collect();
        sr_g_polygon(exterior, interiors.make_array())
    }

    /// Create a MultiPolygon geometry value from polygon values
    ///
    /// `polys` is an array of `len` pointers to values made by
    /// `sr_value_polygon` or `sr_value_polygon_rings`. Each is copied, so the
    /// caller keeps ownership and must still release them with
    /// `sr_value_free`.
    ///
    /// `sr_value_multipolygon` takes flat exterior rings and cannot express a
    /// hole in any of its members; use this when any of them has one. A null
    /// pointer or a non-positive length yields an empty multipolygon.
    ///
    /// Every member must be a Polygon value. If any is null or of another kind
    /// the result is SR_VALUE_NONE rather than a malformed multipolygon.
    ///
    /// Free with sr_value_free
    ///
    /// # Safety
    ///
    /// - `polys` must be null, or point to at least `len` valid Value pointers
    #[export_name = "sr_value_multipolygon_from"]
    pub extern "C" fn value_multipolygon_from(
        polys: *const *const Value,
        len: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::{sr_g_multipolygon, sr_g_polygon, sr_geometry};

        let wrap = |members: Vec<sr_g_polygon>| {
            Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
                sr_geometry::SR_GEOMETRY_MULTIPOLYGON(sr_g_multipolygon(members.make_array())),
            )))
        };

        if polys.is_null() || len <= 0 {
            return wrap(Vec::new());
        }

        let ptrs = unsafe { std::slice::from_raw_parts(polys, len as usize) };
        let mut members: Vec<sr_g_polygon> = Vec::with_capacity(ptrs.len());
        for &p in ptrs {
            if p.is_null() {
                return Box::into_raw(Box::new(Value::SR_VALUE_NONE));
            }
            match unsafe { &*p } {
                Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_POLYGON(poly)) => {
                    members.push(poly.clone())
                }
                _ => return Box::into_raw(Box::new(Value::SR_VALUE_NONE)),
            }
        }
        wrap(members)
    }

    /// Create a GeometryCollection value from geometry values
    ///
    /// `geoms` is an array of `len` pointers to values made by the other
    /// geometry constructors (`sr_value_point`, `sr_value_polygon`, and so on).
    /// Each is copied, so the caller keeps ownership of the values it passed in
    /// and must still release them with `sr_value_free`. A null pointer or a
    /// non-positive length yields an empty collection.
    ///
    /// Every member must be a geometry value. If any is null or of another
    /// kind the result is SR_VALUE_NONE rather than a malformed collection.
    ///
    /// This is the only supported way to build a collection. Assigning to the
    /// `sr_geometry_collection` union member directly is not: that storage is
    /// released with Rust's allocator, so a block from `malloc` corrupts the
    /// heap when the value is freed.
    ///
    /// Free with sr_value_free
    ///
    /// # Safety
    ///
    /// - `geoms` must be null, or point to at least `len` valid Value pointers
    #[export_name = "sr_value_collection"]
    pub extern "C" fn value_collection(
        geoms: *const *const Value,
        len: std::ffi::c_int,
    ) -> *mut Value {
        use crate::array::MakeArray;
        use crate::geometry::sr_geometry;

        let collection = |members: Vec<sr_geometry>| {
            Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(
                sr_geometry::SR_GEOMETRY_COLLECTION(members.make_array()),
            )))
        };

        if geoms.is_null() || len <= 0 {
            return collection(Vec::new());
        }

        let ptrs = unsafe { std::slice::from_raw_parts(geoms, len as usize) };
        let mut members: Vec<sr_geometry> = Vec::with_capacity(ptrs.len());
        for &p in ptrs {
            if p.is_null() {
                return Box::into_raw(Box::new(Value::SR_VALUE_NONE));
            }
            match unsafe { &*p } {
                Value::SR_GEOMETRY_OBJECT(g) => members.push(g.clone()),
                _ => return Box::into_raw(Box::new(Value::SR_VALUE_NONE)),
            }
        }
        collection(members)
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
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_POINT(point))))
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
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_LINESTRING(ls))));
        }
        
        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let vec: Vec<crate::geometry::sr_g_coord> = slice.to_vec();
        let ls = sr_g_linestring(vec.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_LINESTRING(ls))))
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
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_POLYGON(poly))));
        }
        
        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let vec: Vec<crate::geometry::sr_g_coord> = slice.to_vec();
        let exterior = sr_g_linestring(vec.make_array());
        let interiors: Vec<sr_g_linestring> = Vec::new();
        let poly = sr_g_polygon(exterior, interiors.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_POLYGON(poly))))
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
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_MULTIPOINT(mp))));
        }
        
        let slice = unsafe { std::slice::from_raw_parts(coords, len as usize) };
        let points: Vec<sr_g_point> = slice.iter()
            .map(|c| sr_g_point(sr_g_coord { x: c.x, y: c.y }))
            .collect();
        let mp = sr_g_multipoint(points.make_array());
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_MULTIPOINT(mp))))
    }

    /// Create a MultiLineString geometry value
    /// linestrings is an array of pointers to coordinate arrays
    /// lens is an array of lengths for each linestring
    /// count is the number of linestrings
    #[export_name = "sr_value_multilinestring"]
    pub extern "C" fn value_multilinestring(
        linestrings: *const *const crate::geometry::sr_g_coord,
        lens: *const std::ffi::c_int,
        count: std::ffi::c_int
    ) -> *mut Value {
        use crate::geometry::{sr_g_linestring, sr_g_multilinestring, sr_geometry};
        use crate::array::MakeArray;
        
        if linestrings.is_null() || lens.is_null() || count <= 0 {
            let mls = sr_g_multilinestring(Vec::<sr_g_linestring>::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_MULTILINE(mls))));
        }
        
        let linestring_ptrs = unsafe { std::slice::from_raw_parts(linestrings, count as usize) };
        let lengths = unsafe { std::slice::from_raw_parts(lens, count as usize) };
        
        let lines: Vec<sr_g_linestring> = linestring_ptrs.iter()
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
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_MULTILINE(mls))))
    }

    /// Create a MultiPolygon geometry value
    /// polygons is an array of pointers to coordinate arrays (exterior rings only)
    /// lens is an array of lengths for each polygon's exterior ring
    /// count is the number of polygons
    #[export_name = "sr_value_multipolygon"]
    pub extern "C" fn value_multipolygon(
        polygons: *const *const crate::geometry::sr_g_coord,
        lens: *const std::ffi::c_int,
        count: std::ffi::c_int
    ) -> *mut Value {
        use crate::geometry::{sr_g_linestring, sr_g_polygon, sr_g_multipolygon, sr_geometry};
        use crate::array::MakeArray;
        
        if polygons.is_null() || lens.is_null() || count <= 0 {
            let mpoly = sr_g_multipolygon(Vec::<sr_g_polygon>::new().make_array());
            return Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_MULTIPOLYGON(mpoly))));
        }
        
        let polygon_ptrs = unsafe { std::slice::from_raw_parts(polygons, count as usize) };
        let lengths = unsafe { std::slice::from_raw_parts(lens, count as usize) };
        
        let polys: Vec<sr_g_polygon> = polygon_ptrs.iter()
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
        Box::into_raw(Box::new(Value::SR_GEOMETRY_OBJECT(sr_geometry::SR_GEOMETRY_MULTIPOLYGON(mpoly))))
    }

    /// Create a Decimal value from string representation
    #[export_name = "sr_value_decimal"]
    pub extern "C" fn value_decimal(val: *const std::ffi::c_char) -> *mut Value {
        let s = unsafe { std::ffi::CStr::from_ptr(val) }
            .to_string_lossy()
            .to_string()
            .to_string_t();
        Box::into_raw(Box::new(Value::SR_VALUE_NUMBER(Number::SR_NUMBER_DECIMAL(s))))
    }
}
