//!
//! ```rust
//! use geo_types::{Geometry, Point};
//! use pyo3::{prepare_freethreaded_python, Python};
//! use py_geo_interface::GeoInterface;
//!
//! prepare_freethreaded_python();
//!
//! let geom = Python::with_gil(|py| {
//!
//!     // Define a python class implementing the geo_interface. This could also be a shapely or geojson
//!     // object instead. These provide the same interface.
//!     py.run(r#"
//! class Something:
//!     @property
//!     def __geo_interface__(self):
//!          return {"type": "Point", "coordinates": [5., 3.]}
//! "#, None, None).unwrap();
//!
//!     // create an instance of the class and extract the geometry
//!     py.eval(r#"Something()"#, None, None)?.extract::<GeoInterface>()
//! }).unwrap();
//! assert_eq!(geom.0, Geometry::Point(Point::new(5.0_f64, 3.0_f64)));
//! ```
//!
//! Pass geometries from Rust to Python:
//!
//! ```rust
//! use geo_types::{Geometry, Point};
//! use pyo3::{prepare_freethreaded_python, Python};
//! use pyo3::types::{PyDict, PyTuple};
//! use pyo3::IntoPy;
//! use py_geo_interface::GeoInterface;
//!
//! prepare_freethreaded_python();
//!
//! Python::with_gil(|py| {
//!
//!     let geom: GeoInterface = Geometry::Point(Point::new(10.6_f64, 23.3_f64)).into();
//!     let mut locals = PyDict::new(py);
//!     locals.set_item("geom", geom.into_py(py)).unwrap();
//!
//!     py.run(r#"
//! assert geom.__geo_interface__["type"] == "Point"
//! assert geom.__geo_interface__["coordinates"] == (10.6, 23.3)
//! "#, None, Some(locals)).unwrap();
//! });
//! ```

pub mod from_py;
pub mod to_py;

use crate::to_py::GeoAsPyDict;
use from_py::AsGeometry;
use geo_types::Geometry;
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[derive(Debug)]
#[pyclass]
pub struct GeoInterface(pub Geometry<f64>);

#[pymethods]
impl GeoInterface {
    #[getter]
    fn __geo_interface__<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        self.0.geometry_as_pydict(py)
    }
}

impl<'source> FromPyObject<'source> for GeoInterface {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(GeoInterface(ob.as_geometry()?))
    }
}

impl From<Geometry<f64>> for GeoInterface {
    fn from(geom: Geometry<f64>) -> Self {
        Self(geom)
    }
}

macro_rules! geometry_enum_from_impl {
    ($geom_type:ty, $enum_variant_name:ident) => {
        impl From<$geom_type> for GeoInterface {
            fn from(g: $geom_type) -> Self {
                GeoInterface(geo_types::Geometry::$enum_variant_name(g))
            }
        }
    };
}
geometry_enum_from_impl!(geo_types::Point<f64>, Point);
geometry_enum_from_impl!(geo_types::MultiPoint<f64>, MultiPoint);
geometry_enum_from_impl!(geo_types::LineString<f64>, LineString);
geometry_enum_from_impl!(geo_types::MultiLineString<f64>, MultiLineString);
geometry_enum_from_impl!(geo_types::Polygon<f64>, Polygon);
geometry_enum_from_impl!(geo_types::MultiPolygon<f64>, MultiPolygon);
geometry_enum_from_impl!(geo_types::GeometryCollection<f64>, GeometryCollection);
geometry_enum_from_impl!(geo_types::Rect<f64>, Rect);
geometry_enum_from_impl!(geo_types::Line<f64>, Line);
geometry_enum_from_impl!(geo_types::Triangle<f64>, Triangle);

impl From<GeoInterface> for Geometry<f64> {
    fn from(gi: GeoInterface) -> Self {
        gi.0
    }
}
