use std::fmt::Debug;
use surrealdb::sql::Geometry;
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

impl From<sr_g_linestring> for LineString<f64> {
    fn from(l: sr_g_linestring) -> Self {
        LineString::new(l.0.as_slice().iter().map(|c| Coord::from(c)).collect())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
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

impl From<sr_g_polygon> for Polygon<f64> {
    fn from(p: sr_g_polygon) -> Self {
        Polygon::new(
            p.0.into(),
            p.1.as_slice().iter().cloned().map(|l| l.into()).collect(),
        )
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

impl From<sr_g_multipoint> for MultiPoint<f64> {
    fn from(m: sr_g_multipoint) -> Self {
        MultiPoint::from(m.0.as_slice().iter().cloned().map(|p| p.into()).collect::<Vec<Point<f64>>>())
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

impl From<sr_g_multilinestring> for MultiLineString<f64> {
    fn from(m: sr_g_multilinestring) -> Self {
        MultiLineString::new(m.0.as_slice().iter().cloned().map(|l| l.into()).collect())
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

impl From<sr_g_multipolygon> for MultiPolygon<f64> {
    fn from(m: sr_g_multipolygon) -> Self {
        MultiPolygon::new(m.0.as_slice().iter().cloned().map(|p| p.into()).collect())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum sr_geometry {
    sr_g_point(sr_g_point),
    sr_g_linestring(sr_g_linestring),
    sr_g_polygon(sr_g_polygon),
    sr_g_multipoint(sr_g_multipoint),
    sr_g_multiline(sr_g_multilinestring),
    sr_g_multipolygon(sr_g_multipolygon),
    sr_g_collection(ArrayGen<sr_geometry>),
}

impl From<sr_geometry> for Geometry {
    fn from(g: sr_geometry) -> Self {
        match g {
            sr_geometry::sr_g_point(p) => Geometry::Point(p.into()),
            sr_geometry::sr_g_linestring(l) => Geometry::Line(l.into()),
            sr_geometry::sr_g_polygon(p) => Geometry::Polygon(p.into()),
            sr_geometry::sr_g_multipoint(m) => Geometry::MultiPoint(m.into()),
            sr_geometry::sr_g_multiline(l) => Geometry::MultiLine(l.into()),
            sr_geometry::sr_g_multipolygon(p) => Geometry::MultiPolygon(p.into()),
            sr_geometry::sr_g_collection(c) => Geometry::Collection(
                c.as_slice().iter().cloned().map(|g| Geometry::from(g)).collect()
            ),
        }
    }
}

impl From<Geometry> for sr_geometry {
    fn from(value: Geometry) -> Self {
        match value {
            Geometry::Point(p) => sr_geometry::sr_g_point(p.into()),
            Geometry::Line(l) => sr_geometry::sr_g_linestring(l.into()),
            Geometry::Polygon(p) => sr_geometry::sr_g_polygon(p.into()),
            Geometry::MultiPoint(p) => sr_geometry::sr_g_multipoint(p.into()),
            Geometry::MultiLine(l) => sr_geometry::sr_g_multiline(l.into()),
            Geometry::MultiPolygon(p) => sr_geometry::sr_g_multipolygon(p.into()),
            Geometry::Collection(c) => sr_geometry::sr_g_collection(
                c.into_iter().map(|g| g.into()).collect::<Vec<sr_geometry>>().make_array()
            ),
            _ => unimplemented!("New geometry variants added to SurrealDB must be added here."),
        }
    }
}

impl From<Value> for sr_geometry {
    fn from(value: Value) -> Self {
        match value {
            Value::SR_GEOMETRY_OBJECT(g) => g,
            _ => panic!("Expected SR_GEOMETRY_OBJECT, got {:?}", value),
        }
    }
}
