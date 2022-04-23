use geo_types::{
    Coordinate, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon,
};
use pyo3::types::{PyDict, PyTuple};
use pyo3::{IntoPy, PyObject, PyResult, Python, ToPyObject};
use std::iter::once;

pub trait GeoAsPyDict {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict>;
}

impl GeoAsPyDict for Geometry<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        match self {
            Geometry::Point(g) => g.geometry_as_pydict(py),
            Geometry::Line(g) => g.geometry_as_pydict(py),
            Geometry::LineString(g) => g.geometry_as_pydict(py),
            Geometry::Polygon(g) => g.geometry_as_pydict(py),
            Geometry::MultiPoint(g) => g.geometry_as_pydict(py),
            Geometry::MultiLineString(g) => g.geometry_as_pydict(py),
            Geometry::MultiPolygon(g) => g.geometry_as_pydict(py),
            Geometry::GeometryCollection(g) => g.geometry_as_pydict(py),
            Geometry::Rect(g) => g.to_polygon().geometry_as_pydict(py),
            Geometry::Triangle(g) => g.to_polygon().geometry_as_pydict(py),
        }
    }
}

impl GeoAsPyDict for Point<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(py, "Point", self.0.to_py(py))
    }
}

impl GeoAsPyDict for MultiPoint<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(
            py,
            "MultiPoint",
            PyTuple::new(py, self.0.iter().map(|pt| pt.0.to_py(py))).to_object(py),
        )
    }
}

impl GeoAsPyDict for LineString<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(py, "LineString", self.0.to_py(py))
    }
}

impl GeoAsPyDict for MultiLineString<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(
            py,
            "MultiLineString",
            PyTuple::new(py, self.0.iter().map(|linestring| linestring.0.to_py(py))).to_object(py),
        )
    }
}

impl GeoAsPyDict for Line<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(
            py,
            "LineString",
            PyTuple::new(py, [self.start.to_py(py), self.end.to_py(py)]).to_object(py),
        )
    }
}

fn polygon_coordinates_to_pyobject(py: Python, polygon: &Polygon<f64>) -> PyObject {
    let linestring_objs: Vec<_> = once(polygon.exterior().0.to_py(py))
        .chain(polygon.interiors().iter().map(|ls| ls.0.to_py(py)))
        .collect();
    PyTuple::new(py, linestring_objs).to_object(py)
}

impl GeoAsPyDict for Polygon<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(py, "Polygon", polygon_coordinates_to_pyobject(py, self))
    }
}

impl GeoAsPyDict for MultiPolygon<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        make_geom_dict(
            py,
            "MultiPolygon",
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

fn make_geom_dict<'py>(
    py: Python<'py>,
    geom_type: &str,
    coordinates: PyObject,
) -> PyResult<&'py PyDict> {
    let dict = PyDict::new(py);
    dict.set_item("type", geom_type)?;
    dict.set_item("coordinates", coordinates)?;
    Ok(dict)
}

impl GeoAsPyDict for GeometryCollection<f64> {
    fn geometry_as_pydict<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let dict = PyDict::new(py);
        dict.set_item("type", "GeometryCollection")?;
        dict.set_item(
            "geometries",
            PyTuple::new(
                py,
                self.0
                    .iter()
                    .map(|geom| geom.geometry_as_pydict(py))
                    .collect::<PyResult<Vec<_>>>()?,
            ),
        )?;
        Ok(dict)
    }
}

trait ToPy {
    fn to_py(&self, py: Python) -> PyObject;
}

impl ToPy for Coordinate<f64> {
    fn to_py(&self, py: Python) -> PyObject {
        PyTuple::new(py, &[self.x.into_py(py), self.y.into_py(py)]).to_object(py)
    }
}

impl ToPy for [Coordinate<f64>] {
    fn to_py(&self, py: Python) -> PyObject {
        PyTuple::new(py, self.iter().map(|c| c.to_py(py))).to_object(py)
    }
}
