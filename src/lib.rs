//! Exchange vector geometries between Rust and Python using [pyo3](https://pyo3.rs) and [Pythons `__geo_interface__` protocol](https://gist.github.com/sgillies/2217756).
//!
//! The `__geo_interface__` protocol is implemented by most popular geospatial python modules like `shapely`, `geojson`, `geopandas`, ....
//! While this protocol also defines `Features` and `FeatureCollections`, this library so far only focuses on the `Geometry` type, as
//! this one can be directly mapped to the types of the `geo-types` crate.
//!
//! The main struct of this crate is [`Geometry`]. This the docs there for usage examples.
//!
//! ## Features
//!
//! As rust types exposed to python may not have generic type parameters, there are multiple implementations of the `Geometry` type based
//! on different types for the coordinate values. The default is `f64`, other types can be enabled using the `f32`, `u8`, `u16`, `u32`, `u64`,
//! `i8`, `i16`, `i32` and `i64` feature gates. The implementation are then available as `py_geo_interface::wrappers::[datatype]::Geometry`.
//! The default and probably most common used `f64`-variant is also available as `py_geo_interface::Geometry`.
//!
//! The `wkb` feature adds support for exchanging geometries using the Well-Known-Binary format. The `wkb`-property of `shapely`
//! geometries will be used when found. Additionally, the `Geometry`-type exposed to python will have a `wkb`-property
//! itself. WKB is only supported for the `f64`-variant of the `Geometry`, the feature is disabled per default.
//!
//! ## Examples
//!
//! ### Read python types implementing `__geo_interface__` into `geo-types`:
//!
//! #[include]
//! ```rust
//! use geo_types::{Geometry as GtGeometry, Point};
//! use pyo3::{prepare_freethreaded_python, Python};
//! use pyo3::types::PyDictMethods;
//! use pyo3::types::PyAnyMethods;
//! use py_geo_interface::Geometry;
//!
//! prepare_freethreaded_python();
//!
//! let geom = Python::with_gil(|py| {
//!
//!     // Define a python class implementing the geo_interface. This could also be a shapely or geojson
//!     // object instead. These provide the same interface.
//!     py.run_bound(r#"
//! class Something:
//!     @property
//!     def __geo_interface__(self):
//!          return {"type": "Point", "coordinates": [5., 3.]}
//! "#, None, None).unwrap();
//!
//!     // create an instance of the class and extract the geometry
//!     py.eval_bound(r#"Something()"#, None, None)?.extract::<Geometry>()
//! }).unwrap();
//! assert_eq!(geom.0, GtGeometry::Point(Point::new(5.0_f64, 3.0_f64)));
//! ```
//!
//! ### Pass geometries from Rust to Python:
//!
//! ```rust
//! use geo_types::{Geometry as GtGeometry, Point};
//! use pyo3::{prepare_freethreaded_python, Python};
//! use pyo3::types::{PyDict, PyTuple, PyDictMethods};
//! use pyo3::IntoPy;
//! use py_geo_interface::Geometry;
//!
//! prepare_freethreaded_python();
//!
//! Python::with_gil(|py| {
//!
//!     let geom: Geometry = Point::new(10.6_f64, 23.3_f64).into();
//!     let mut locals = PyDict::new_bound(py);
//!     locals.set_item("geom", geom.into_py(py)).unwrap();
//!
//!     py.run_bound(r#"
//! assert geom.__geo_interface__["type"] == "Point"
//! assert geom.__geo_interface__["coordinates"] == (10.6, 23.3)
//! "#, None, Some(&locals)).unwrap();
//! });
//! ```

pub mod from_py;
pub mod to_py;
pub mod wrappers;

#[cfg(feature = "wkb")]
pub mod wkb;

use crate::from_py::{ExtractFromPyFloat, ExtractFromPyInt};
#[cfg(feature = "wkb")]
use crate::wkb::WKBSupport;
use geo_types::CoordNum;
use pyo3::prelude::*;

#[cfg(feature = "wkb")]
pub trait PyCoordNum:
    CoordNum + IntoPy<Py<PyAny>> + ExtractFromPyFloat + ExtractFromPyInt + WKBSupport
{
}

#[cfg(not(feature = "wkb"))]
pub trait PyCoordNum: CoordNum + IntoPy<Py<PyAny>> + ExtractFromPyFloat + ExtractFromPyInt {}

#[cfg(feature = "wkb")]
impl<T: CoordNum + IntoPy<Py<PyAny>> + ExtractFromPyFloat + ExtractFromPyInt + WKBSupport>
    PyCoordNum for T
{
}

#[cfg(not(feature = "wkb"))]
impl<T: CoordNum + IntoPy<Py<PyAny>> + ExtractFromPyFloat + ExtractFromPyInt> PyCoordNum for T {}

#[cfg(feature = "f64")]
pub use crate::wrappers::f64::Geometry;
#[cfg(feature = "f64")]
pub use crate::wrappers::f64::GeometryVec;
#[cfg(feature = "f64")]
pub use crate::wrappers::f64::GeometryVecFc;
