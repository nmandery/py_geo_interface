use crate::PyCoordNum;
use geo_types::{
    Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon,
};
use pyo3::types::{PyDict, PyString, PyTuple};
use pyo3::{intern, PyObject, PyResult, Python, ToPyObject};
use std::iter::once;

/// Convert `self` to a Python dictionary reflecting the structure of a `__geo_interface__` python dict.
pub trait AsGeoInterface {
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject>;
}

impl<T> AsGeoInterface for Geometry<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        match self {
            Geometry::Point(g) => g.as_geointerface_pyobject(py),
            Geometry::Line(g) => g.as_geointerface_pyobject(py),
            Geometry::LineString(g) => g.as_geointerface_pyobject(py),
            Geometry::Polygon(g) => g.as_geointerface_pyobject(py),
            Geometry::MultiPoint(g) => g.as_geointerface_pyobject(py),
            Geometry::MultiLineString(g) => g.as_geointerface_pyobject(py),
            Geometry::MultiPolygon(g) => g.as_geointerface_pyobject(py),
            Geometry::GeometryCollection(g) => g.as_geointerface_pyobject(py),
            Geometry::Rect(g) => g.to_polygon().as_geointerface_pyobject(py),
            Geometry::Triangle(g) => g.to_polygon().as_geointerface_pyobject(py),
        }
    }
}

impl<T> AsGeoInterface for Point<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(py, intern!(py, "Point"), self.0.to_py(py))
    }
}

impl<T> AsGeoInterface for MultiPoint<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(
            py,
            intern!(py, "MultiPoint"),
            PyTuple::new(py, self.0.iter().map(|pt| pt.0.to_py(py))).to_object(py),
        )
    }
}

impl<T> AsGeoInterface for LineString<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(py, intern!(py, "LineString"), self.0.to_py(py))
    }
}

impl<T> AsGeoInterface for MultiLineString<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(
            py,
            intern!(py, "MultiLineString"),
            PyTuple::new(py, self.0.iter().map(|linestring| linestring.0.to_py(py))).to_object(py),
        )
    }
}

impl<T> AsGeoInterface for Line<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(
            py,
            intern!(py, "LineString"),
            PyTuple::new(py, [self.start.to_py(py), self.end.to_py(py)]).to_object(py),
        )
    }
}

fn polygon_coordinates_to_pyobject<T>(py: Python, polygon: &Polygon<T>) -> PyObject
where
    T: PyCoordNum,
{
    let linestring_objs: Vec<_> = once(polygon.exterior().0.to_py(py))
        .chain(polygon.interiors().iter().map(|ls| ls.0.to_py(py)))
        .collect();
    PyTuple::new(py, linestring_objs).to_object(py)
}

impl<T> AsGeoInterface for Polygon<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(
            py,
            intern!(py, "Polygon"),
            polygon_coordinates_to_pyobject(py, self),
        )
    }
}

impl<T> AsGeoInterface for MultiPolygon<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        make_geom_pyobject(
            py,
            intern!(py, "MultiPolygon"),
            PyTuple::new(
                py,
                self.0
                    .iter()
                    .map(|polygon| polygon_coordinates_to_pyobject(py, polygon)),
            )
            .to_object(py),
        )
    }
}

fn make_geom_pyobject<'py>(
    py: Python<'py>,
    geom_type: &'py PyString,
    coordinates: PyObject,
) -> PyResult<PyObject> {
    let dict = PyDict::new(py);
    dict.set_item(intern!(py, "type"), geom_type)?;
    dict.set_item(intern!(py, "coordinates"), coordinates)?;
    Ok(dict.to_object(py))
}

impl<T> AsGeoInterface for GeometryCollection<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item(intern!(py, "type"), intern!(py, "GeometryCollection"))?;
        dict.set_item(
            intern!(py, "geometries"),
            PyTuple::new(
                py,
                self.0
                    .iter()
                    .map(|geom| geom.as_geointerface_pyobject(py))
                    .collect::<PyResult<Vec<_>>>()?,
            ),
        )?;
        Ok(dict.to_object(py))
    }
}

trait ToPy {
    fn to_py(&self, py: Python) -> PyObject;
}

impl<T> ToPy for Coordinate<T>
where
    T: PyCoordNum,
{
    fn to_py(&self, py: Python) -> PyObject {
        PyTuple::new(py, &[self.x.into_py(py), self.y.into_py(py)]).to_object(py)
    }
}

impl<T> ToPy for [Coordinate<T>]
where
    T: PyCoordNum,
{
    fn to_py(&self, py: Python) -> PyObject {
        PyTuple::new(py, self.iter().map(|c| c.to_py(py))).to_object(py)
    }
}
