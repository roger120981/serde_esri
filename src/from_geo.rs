// This module is for converting geo types into Esri geometry objects
// requires geo for cw and ccw enforcement

use crate::geometry::*;
use geo::{CoordsIter, Winding};
use geo_types::{
    Coord, Geometry, Line, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
    Rect, Triangle,
};
use std::iter::once;

impl Into<EsriCoord<2>> for Coord {
    fn into(self) -> EsriCoord<2> {
        EsriCoord::<2>([self.x, self.y])
    }
}

impl Into<EsriPoint> for Point {
    fn into(self) -> EsriPoint {
        EsriPoint {
            x: self.x(),
            y: self.y(),
            z: None,
            m: None,
            spatialReference: None,
        }
    }
}

impl Into<EsriMultiPoint<2>> for MultiPoint {
    fn into(self) -> EsriMultiPoint<2> {
        let coords = make_coord_vec(self.coords_iter());

        EsriMultiPoint {
            hasZ: None,
            hasM: None,
            points: coords,
            spatialReference: None,
        }
    }
}

impl Into<EsriPolyline<2>> for Line {
    fn into(self) -> EsriPolyline<2> {
        let coords: Vec<EsriCoord<2>> = vec![self.start.into(), self.end.into()];

        EsriPolyline {
            hasZ: None,
            hasM: None,
            paths: vec![EsriLineString::<2>(coords)],
            spatialReference: None,
        }
    }
}

impl Into<EsriPolyline<2>> for LineString {
    fn into(self) -> EsriPolyline<2> {
        let coords = make_coord_vec(self.coords_iter());

        EsriPolyline {
            hasZ: None,
            hasM: None,
            paths: vec![EsriLineString::<2>(coords)],
            spatialReference: None,
        }
    }
}

impl Into<EsriPolyline<2>> for MultiLineString {
    fn into(self) -> EsriPolyline<2> {
        let lns = self
            .iter()
            .map(|ln| {
                let coords = make_coord_vec(ln.coords_iter());
                EsriLineString::<2>(coords)
            })
            .collect::<Vec<EsriLineString<2>>>();

        EsriPolyline {
            hasZ: None,
            hasM: None,
            paths: lns,
            spatialReference: None,
        }
    }
}

impl Into<EsriPolygon<2>> for Polygon {
    fn into(self) -> EsriPolygon<2> {
        // ensure that the exterior ring is clockwise
        let exterior = if self.exterior().is_ccw() {
            EsriLineString::<2>(make_coord_vec_rev(self.exterior().coords_iter()))
        } else {
            EsriLineString::<2>(make_coord_vec(self.exterior().coords_iter()))
        };

        // ensure interior rings are counter-clockwise
        let interiors = self.interiors().iter().map(|i| {
            if i.is_cw() {
                EsriLineString::<2>(make_coord_vec_rev(i.coords_iter()))
            } else {
                EsriLineString::<2>(make_coord_vec(i.coords_iter()))
            }
        });

        let rings: Vec<EsriLineString<2>> = once(exterior).chain(interiors).collect();

        EsriPolygon {
            hasZ: None,
            hasM: None,
            rings: rings,
            spatialReference: None,
        }
    }
}

impl Into<EsriPolygon<2>> for MultiPolygon {
    fn into(self) -> EsriPolygon<2> {
        let rings: Vec<EsriLineString<2>> = self
            .iter()
            .flat_map(|poly| {
                // esure that the exterior ring is clockwise
                let exterior = if poly.exterior().is_ccw() {
                    EsriLineString::<2>(make_coord_vec_rev(poly.exterior().coords_iter()))
                } else {
                    EsriLineString::<2>(make_coord_vec(poly.exterior().coords_iter()))
                };

                // ensure interior rings are counter-clockwise
                let interiors = poly.interiors().iter().map(|i| {
                    if i.is_cw() {
                        EsriLineString::<2>(make_coord_vec_rev(i.coords_iter()))
                    } else {
                        EsriLineString::<2>(make_coord_vec(i.coords_iter()))
                    }
                });

                once(exterior).chain(interiors)
            })
            .collect();

        EsriPolygon {
            hasZ: None,
            hasM: None,
            rings: rings,
            spatialReference: None,
        }
    }
}

impl Into<EsriPolygon<2>> for Triangle {
    fn into(self) -> EsriPolygon<2> {
        self.to_polygon().into()
    }
}

impl Into<EsriPolygon<2>> for Rect {
    fn into(self) -> EsriPolygon<2> {
        self.to_polygon().into()
    }
}

impl TryInto<EsriGeometry<2>> for Geometry {
    type Error = Option<()>;

    fn try_into(self) -> Result<EsriGeometry<2>, Self::Error> {
        match self {
            Geometry::Point(g) => Ok(EsriGeometry::Point(g.into())),
            Geometry::MultiPoint(g) => Ok(EsriGeometry::MultiPoint(g.into())),

            Geometry::Line(g) => Ok(EsriGeometry::Polyline(g.into())),
            Geometry::LineString(g) => Ok(EsriGeometry::Polyline(g.into())),
            Geometry::MultiLineString(g) => Ok(EsriGeometry::Polyline(g.into())),

            Geometry::Polygon(g) => Ok(EsriGeometry::Polygon(g.into())),
            Geometry::MultiPolygon(g) => Ok(EsriGeometry::Polygon(g.into())),
            Geometry::Rect(g) => Ok(EsriGeometry::Polygon(g.into())),
            Geometry::Triangle(g) => Ok(EsriGeometry::Polygon(g.into())),

            Geometry::GeometryCollection(_g) => Err(None),
        }
    }
}

fn make_coord_vec(coords: impl Iterator<Item = impl Into<EsriCoord<2>>>) -> Vec<EsriCoord<2>> {
    coords
        .map(Into::<EsriCoord<2>>::into)
        .collect::<Vec<EsriCoord<2>>>()
}

fn make_coord_vec_rev(
    coords: impl DoubleEndedIterator<Item = impl Into<EsriCoord<2>>>,
) -> Vec<EsriCoord<2>> {
    coords
        .rev()
        .map(Into::<EsriCoord<2>>::into)
        .collect::<Vec<EsriCoord<2>>>()
}
