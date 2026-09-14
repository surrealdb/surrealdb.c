use std::fmt::Debug;
use surrealdb::types::Geometry;
use geo_types::{Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon, Coord};
use crate::array::*;
use crate::value::Value;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_coord {
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for sr_g_coord {
    fn from(c: (f64, f64)) -> Self {
        Self {
            x: c.0,
            y: c.1,
        }
    }
}

impl From<Coord<f64>> for sr_g_coord {
    fn from(c: Coord<f64>) -> Self {
        sr_g_coord {
            x: c.x,
            y: c.y,
        }
    }
}

impl From<sr_g_coord> for Coord<f64> {
    fn from(val: sr_g_coord) -> Self {
        Coord {
            x: val.x,
            y: val.y,
        }
    }
}

impl From<&sr_g_coord> for Coord<f64> {
    fn from(val: &sr_g_coord) -> Self {
        Coord {
            x: val.x,
            y: val.y,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_point(pub sr_g_coord);

impl From<Point<f64>> for sr_g_point {
    fn from(p: Point<f64>) -> Self {
        sr_g_point(sr_g_coord {
            x: p.x(),
            y: p.y()
        })
    } 
}

impl From<sr_g_point> for Point<f64> {
    fn from(p: sr_g_point) -> Self {
        Point::new(p.0.x, p.0.y)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_linestring(pub ArrayGen<sr_g_coord>);

impl From<LineString<f64>> for sr_g_linestring {
    fn from(l: LineString<f64>) -> Self {
        let v = l.0.into_iter().map(|c| c.into()).collect::<Vec<sr_g_coord>>();
        sr_g_linestring(v.make_array())
    }
}

impl From<&sr_g_linestring> for LineString<f64> {
    fn from(l: &sr_g_linestring) -> Self {
        LineString::new(l.0.as_slice().iter().map(|c| Coord::from(c)).collect())
    }
}

impl From<sr_g_linestring> for LineString<f64> {
    /// Borrows and then lets `l` drop, rather than moving its array out. Moving
    /// a field out of a type with a destructor is E0509, and there is nothing
    /// here worth moving: the conversion only reads coordinates.
    fn from(l: sr_g_linestring) -> Self {
        LineString::from(&l)
    }
}

impl Drop for sr_g_linestring {
    fn drop(&mut self) {
        self.0.free();
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
/// A polygon: an exterior ring, then zero or more interior rings (holes).
///
/// The interiors are owned by the value and released with Rust's allocator.
/// Do not assign to that field from C: a block from `malloc` is freed with the
/// wrong allocator and corrupts the heap. Build polygons with
/// `sr_value_polygon_rings`, which copies -- `sr_value_polygon` takes a single
/// ring and cannot express a hole.
pub struct sr_g_polygon(pub sr_g_linestring, pub ArrayGen<sr_g_linestring>);

impl From<Polygon<f64>> for sr_g_polygon {
    fn from(p: Polygon<f64>) -> Self {
        let (exterior, interiors) = p.into_inner();
        sr_g_polygon(
            exterior.into(),
            interiors.into_iter().map(|l| l.into()).collect::<Vec<sr_g_linestring>>().make_array()
        )
    }
}

impl From<&sr_g_polygon> for Polygon<f64> {
    fn from(p: &sr_g_polygon) -> Self {
        Polygon::new(
            LineString::from(&p.0),
            // Previously `.cloned()`, which deep-copied every interior ring
            // only to convert and discard it -- and, with no destructor on
            // these types, leaked each copy. Borrowing avoids both.
            p.1.as_slice().iter().map(LineString::from).collect(),
        )
    }
}

impl From<sr_g_polygon> for Polygon<f64> {
    fn from(p: sr_g_polygon) -> Self {
        Polygon::from(&p)
    }
}

impl Drop for sr_g_polygon {
    /// Frees the interior rings. The exterior ring is field 0 and is released
    /// by its own destructor once this returns; freeing the ArrayGen drops the
    /// boxed slice, which runs each interior ring's destructor in turn.
    fn drop(&mut self) {
        self.1.free();
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_multipoint(pub ArrayGen<sr_g_point>);

impl From<MultiPoint<f64>> for sr_g_multipoint {
    fn from(m: MultiPoint<f64>) -> Self {
        sr_g_multipoint(m.0.into_iter().map(|p| p.into()).collect::<Vec<sr_g_point>>().make_array())
    }
}

impl From<&sr_g_multipoint> for MultiPoint<f64> {
    fn from(m: &sr_g_multipoint) -> Self {
        MultiPoint::from(
            m.0.as_slice()
                .iter()
                .map(|p| Point::new(p.0.x, p.0.y))
                .collect::<Vec<Point<f64>>>(),
        )
    }
}

impl From<sr_g_multipoint> for MultiPoint<f64> {
    fn from(m: sr_g_multipoint) -> Self {
        MultiPoint::from(&m)
    }
}

impl Drop for sr_g_multipoint {
    /// Points carry an inline coordinate and own nothing, so releasing the
    /// array itself is the whole job.
    fn drop(&mut self) {
        self.0.free();
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_multilinestring(pub ArrayGen<sr_g_linestring>);

impl From<MultiLineString<f64>> for sr_g_multilinestring {
    fn from(m: MultiLineString<f64>) -> Self {
        sr_g_multilinestring(m.0.into_iter().map(|l| l.into()).collect::<Vec<sr_g_linestring>>().make_array())
    }
}

impl From<&sr_g_multilinestring> for MultiLineString<f64> {
    fn from(m: &sr_g_multilinestring) -> Self {
        MultiLineString::new(m.0.as_slice().iter().map(LineString::from).collect())
    }
}

impl From<sr_g_multilinestring> for MultiLineString<f64> {
    fn from(m: sr_g_multilinestring) -> Self {
        MultiLineString::from(&m)
    }
}

impl Drop for sr_g_multilinestring {
    fn drop(&mut self) {
        self.0.free();
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_multipolygon(pub ArrayGen<sr_g_polygon>);

impl From<MultiPolygon<f64>> for sr_g_multipolygon {
    fn from(m: MultiPolygon<f64>) -> Self {
        sr_g_multipolygon(m.0.into_iter().map(|p| p.into()).collect::<Vec<sr_g_polygon>>().make_array())
    }
}

impl From<&sr_g_multipolygon> for MultiPolygon<f64> {
    fn from(m: &sr_g_multipolygon) -> Self {
        MultiPolygon::new(m.0.as_slice().iter().map(Polygon::from).collect())
    }
}

impl From<sr_g_multipolygon> for MultiPolygon<f64> {
    fn from(m: sr_g_multipolygon) -> Self {
        MultiPolygon::from(&m)
    }
}

impl Drop for sr_g_multipolygon {
    fn drop(&mut self) {
        self.0.free();
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum sr_geometry {
    SR_GEOMETRY_POINT(sr_g_point),
    SR_GEOMETRY_LINESTRING(sr_g_linestring),
    SR_GEOMETRY_POLYGON(sr_g_polygon),
    SR_GEOMETRY_MULTIPOINT(sr_g_multipoint),
    SR_GEOMETRY_MULTILINE(sr_g_multilinestring),
    SR_GEOMETRY_MULTIPOLYGON(sr_g_multipolygon),
    /// A heterogeneous collection of geometries.
    ///
    /// This storage is owned by the value and released with Rust's allocator.
    /// Do not assign to the `sr_geometry_collection` union member from C: a
    /// block from `malloc` is freed with the wrong allocator and corrupts the
    /// heap. Build collections with `sr_value_collection`, which copies.
    SR_GEOMETRY_COLLECTION(ArrayGen<sr_geometry>),
    /// Represents a geometry type added in a newer version of SurrealDB
    /// that this C API version doesn't yet support
    SR_GEOMETRY_UNIMPLEMENTED,
}

impl From<&sr_geometry> for Geometry {
    fn from(g: &sr_geometry) -> Self {
        match g {
            sr_geometry::SR_GEOMETRY_POINT(p) => Geometry::Point(Point::new(p.0.x, p.0.y)),
            sr_geometry::SR_GEOMETRY_LINESTRING(l) => Geometry::Line(l.into()),
            sr_geometry::SR_GEOMETRY_POLYGON(p) => Geometry::Polygon(p.into()),
            sr_geometry::SR_GEOMETRY_MULTIPOINT(m) => Geometry::MultiPoint(m.into()),
            sr_geometry::SR_GEOMETRY_MULTILINE(l) => Geometry::MultiLine(l.into()),
            sr_geometry::SR_GEOMETRY_MULTIPOLYGON(p) => Geometry::MultiPolygon(p.into()),
            // Borrowed rather than `.cloned()`: cloning deep-copied every
            // nested geometry just to convert and discard it.
            sr_geometry::SR_GEOMETRY_COLLECTION(c) => {
                Geometry::Collection(c.as_slice().iter().map(Geometry::from).collect())
            }
            sr_geometry::SR_GEOMETRY_UNIMPLEMENTED => Geometry::Point(Point::new(0.0, 0.0)),
        }
    }
}

impl From<sr_geometry> for Geometry {
    /// Reads through a borrow so `g` can carry a destructor; it is released on
    /// return rather than having its payload moved out.
    fn from(g: sr_geometry) -> Self {
        Geometry::from(&g)
    }
}

impl Drop for sr_geometry {
    /// Only the collection variant owns anything the enum itself must release:
    /// every other payload is a struct with its own destructor, which drop glue
    /// runs. Freeing the array drops each nested geometry in turn, so nested
    /// collections unwind correctly.
    fn drop(&mut self) {
        if let sr_geometry::SR_GEOMETRY_COLLECTION(c) = self {
            c.free();
        }
    }
}

impl From<Geometry> for sr_geometry {
    fn from(value: Geometry) -> Self {
        match value {
            Geometry::Point(p) => sr_geometry::SR_GEOMETRY_POINT(p.into()),
            Geometry::Line(l) => sr_geometry::SR_GEOMETRY_LINESTRING(l.into()),
            Geometry::Polygon(p) => sr_geometry::SR_GEOMETRY_POLYGON(p.into()),
            Geometry::MultiPoint(p) => sr_geometry::SR_GEOMETRY_MULTIPOINT(p.into()),
            Geometry::MultiLine(l) => sr_geometry::SR_GEOMETRY_MULTILINE(l.into()),
            Geometry::MultiPolygon(p) => sr_geometry::SR_GEOMETRY_MULTIPOLYGON(p.into()),
            Geometry::Collection(c) => sr_geometry::SR_GEOMETRY_COLLECTION(
                c.into_iter().map(|g| g.into()).collect::<Vec<sr_geometry>>().make_array()
            ),
        }
    }
}

impl From<Value> for sr_geometry {
    fn from(value: Value) -> Self {
        match value {
            Value::SR_GEOMETRY_OBJECT(g) => g,
            _ => sr_geometry::SR_GEOMETRY_UNIMPLEMENTED,
        }
    }
}
