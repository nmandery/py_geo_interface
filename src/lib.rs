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

pub mod from_py;

use from_py::AsGeometry;
use geo_types::Geometry;
use pyo3::{FromPyObject, PyAny, PyResult};

#[derive(Debug)]
pub struct GeoInterface(pub Geometry<f64>);

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

impl From<GeoInterface> for Geometry<f64> {
    fn from(gi: GeoInterface) -> Self {
        gi.0
    }
}
