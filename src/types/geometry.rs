use std::ffi::{c_double, c_float, c_int};
use std::fmt::Debug;
use surrealdb::sql;
use surrealdb::sql::{
    Geometry  
};
use geo_types::{
    Point, 
    LineString, 
    Polygon, 
    MultiPoint, 
    MultiLineString, 
    MultiPolygon, 
    Coord
};
use crate::array::{
    ArrayGen
};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_coord {
    pub x: f64,
    pub y: f64,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_point(pub sr_coord);

impl Into<Point<f64>> for sr_g_point {
    fn into(self) -> Point<f64> {
        Point::new(self.0.x, self.0.y)
    }
}

impl Into<Coord> for sr_coord {
    fn into(self) -> Coord {
        Coord::from((self.x, self.y))
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_linestring(pub ArrayGen<sr_coord>);

impl Debug for ArrayGen<sr_coord> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArrayGen<sr_coord>")
    }  
}

impl PartialEq for ArrayGen<sr_coord> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.clone().into_vec();
        let b = other.clone().into_vec();
        a == b   
    }
}

impl Into<Vec<sr_coord>> for ArrayGen<sr_coord> {
    fn into(self) -> Vec<sr_coord> {
        self.into_vec().into_iter().collect::<Vec<sr_coord>>()
    }
}

impl Into<LineString> for sr_g_linestring {
    fn into(self) -> LineString {
        // This is both beautiful and ugly.
        // What we lose in flexibility we gain in elegance.
        // Some refer to this as magic, and magic can be bad.
        // You may not like it, but this is what peak functional performance looks like.
        // Help me.
        LineString::new(self.0.into_vec().into_iter().map(|c| c.into()).collect::<Vec<Coord<f64>>>())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_polygon(pub sr_g_linestring, pub ArrayGen<sr_g_linestring>);

impl Debug for ArrayGen<sr_g_linestring> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArrayGen<sr_g_linestring>")
    } 
}

impl PartialEq for ArrayGen<sr_g_linestring> {
    fn eq(&self, other: &Self) -> bool {
        let s = self.clone().into_vec();
        let o = other.clone().into_vec();
        s == o  
    }
}

impl Into<Vec<LineString>> for ArrayGen<sr_g_linestring> {
    fn into(self) -> Vec<LineString> {
        self.into_vec().into_iter().map(|c| c.into()).collect::<Vec<LineString<f64>>>()
    }
}

impl Into<Polygon<f64>> for sr_g_polygon {
    fn into(self) -> Polygon<f64> {
        Polygon::new(self.0.into(), self.1.into())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_multipoint(pub ArrayGen<sr_g_point>);

impl Debug for ArrayGen<sr_g_point> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArrayGen<sr_g_point>")
    } 
}

impl PartialEq for ArrayGen<sr_g_point>
{
    fn eq(&self, other: &Self) -> bool {
        let s = self.clone().into_vec();
        let o = other.clone().into_vec();
        s == o 
    }
}

impl Into<Vec<Point<f64>>> for ArrayGen<sr_g_point> {
    fn into(self) -> Vec<Point<f64>> {
        self.into_vec().into_iter().map(|c| c.into()).collect::<Vec<Point<f64>>>()
    }
}

impl Into<MultiPoint<f64>> for sr_g_multipoint {
    fn into(self) -> MultiPoint<f64> {
        MultiPoint::from(self.0.into_vec().into_iter().map(|c| c.into()).collect::<Vec<Point<f64>>>())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_multilinestring(pub ArrayGen<sr_g_linestring>);

impl Into<MultiLineString<f64>> for sr_g_multilinestring {
    fn into(self) -> MultiLineString<f64> {
        MultiLineString::new(self.0.into_vec().into_iter().map(|c| c.into()).collect::<Vec<LineString<f64>>>())
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub struct sr_g_multipolygon(pub ArrayGen<sr_g_polygon>);

impl Debug for ArrayGen<sr_g_polygon> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArrayGen<sr_g_polygon>")
    } 
}

impl PartialEq for ArrayGen<sr_g_polygon> {
    fn eq(&self, other: &Self) -> bool {
        let s = self.clone().into_vec();
        let o = other.clone().into_vec();
        s == o
    }
}

impl Into<Vec<Polygon<f64>>> for ArrayGen<sr_g_polygon> {
    fn into(self) -> Vec<Polygon<f64>> {
        self.into_vec().into_iter().map(|c| c.into()).collect::<Vec<Polygon<f64>>>()
    }
}

impl Into<MultiPolygon<f64>> for sr_g_multipolygon {
    fn into(self) -> MultiPolygon<f64> {
        MultiPolygon::new(self.0.into_vec().into_iter().map(|c| c.into()).collect::<Vec<Polygon<f64>>>())
    }  
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
#[repr(C)]
pub enum sr_geometry {
    sr_g_point(sr_g_point),
    sr_g_line(sr_g_linestring),
    sr_g_polygon(sr_g_polygon),
    sr_g_multipoint(sr_g_multipoint),
    sr_g_multiline(sr_g_multilinestring),
    sr_g_multipolygon(sr_g_multipolygon),
    sr_g_collection(ArrayGen<sr_geometry>),
}

impl Into<Geometry> for sr_geometry {
    fn into(self) -> Geometry {
        match self {
            sr_geometry::sr_g_point(p) => Geometry::Point(p.into()),
            sr_geometry::sr_g_line(l) => {
                Geometry::Line(l.into())
            },
            sr_geometry::sr_g_polygon(p) => Geometry::Polygon(p.into()),
            sr_geometry::sr_g_multipoint(p) => Geometry::MultiPoint(p.into()),
            sr_geometry::sr_g_multiline(l) => Geometry::MultiLine(l.into()),
            sr_geometry::sr_g_multipolygon(p) => Geometry::MultiPolygon(p.into()),
            sr_geometry::sr_g_collection(c) => Geometry::Collection(c.into()),
        }
    }
}

impl Into<Vec<Geometry>> for ArrayGen<sr_geometry> {
    fn into(self) -> Vec<Geometry> {
        let col = self.into_vec();
        let mut geo: Vec<Geometry> = vec![];
        for i in 0..col.len() {
            geo.push(col[i].clone().into());
        };
        
        geo
    }
}

impl Debug for ArrayGen<sr_geometry> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ArrayGen<sr_geometry>")
    }   
}

impl PartialEq for ArrayGen<sr_geometry> {
    fn eq(&self, other: &Self) -> bool {
        let s = self.clone().into_vec();
        let o = other.clone().into_vec();
        s == o 
    }
}


