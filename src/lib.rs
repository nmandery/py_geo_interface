//! Exchange vector geometries between Rust and Python using [pyo3](https://pyo3.rs) and [Pythons `__geo_interface__` protocol](https://gist.github.com/sgillies/2217756).
//!
//! The `__geo_interface__` protocol is implemented by most popular geospatial python modules like `shapely`, `geojson`, `geopandas`, ....
//!
//! The main struct of this crate is [`GeometryInterface`]. This the docs there for usage examples.
//!
//! ## Features
//!
//! As rust types exposed to python may not have generic type parameters, there are multiple implementations of the `GeometryInterface` type based
//! on different types for the coordinate values. The default is `f64`, other types can be enabled using the `f32`, `u8`, `u16`, `u32`, `u64`,
//! `i8`, `i16`, `i32` and `i64` feature gates. The implementation are then available as `py_geo_interface::wrappers::[datatype]::GeometryInterface`.
//! The default and probably most common used `f64`-variant is also available as `py_geo_interface::GeometryInterface`.
//!
//! The `wkb` feature adds support for exchanging geometries using the Well-Known-Binary format. The `wkb`-property of `shapely`
//! geometries will be used when found. Additionally, the `GeometryInterface`-type exposed to python will have a `wkb`-property
//! itself. WKB is only supported for the `f64`-variant of the `GeometryInterface`, the feature is disabled per default.
//!
//! ## Examples
//!
//! ### Read python types implementing `__geo_interface__` into `geo-types`:
//!
//! #[include]
//! ```rust
//! use geo_types::{Geometry, Point};
//! use pyo3::{prepare_freethreaded_python, Python};
//! use py_geo_interface::GeometryInterface;
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
//!     py.eval(r#"Something()"#, None, None)?.extract::<GeometryInterface>()
//! }).unwrap();
//! assert_eq!(geom.0, Geometry::Point(Point::new(5.0_f64, 3.0_f64)));
//! ```
//!
//! ### Pass geometries from Rust to Python:
//!
//! ```rust
//! use geo_types::{Geometry, Point};
//! use pyo3::{prepare_freethreaded_python, Python};
//! use pyo3::types::{PyDict, PyTuple};
//! use pyo3::IntoPy;
//! use py_geo_interface::GeometryInterface;
//!
//! prepare_freethreaded_python();
//!
//! Python::with_gil(|py| {
//!
//!     let geom: GeometryInterface = Point::new(10.6_f64, 23.3_f64).into();
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
pub mod wrappers;

#[cfg(feature = "f64")]
pub mod series;

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
pub use crate::wrappers::f64::GeometryInterface;
