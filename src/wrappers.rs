macro_rules! dt_mod {
    ($coord_type:ty, $mod_name:ident) => {
        pub mod $mod_name {
            use crate::from_py::AsGeometry;
            use crate::to_py::AsGeoInterface;
            use crate::to_py::AsGeoInterfaceFeatureCollection;
            use crate::to_py::AsGeoInterfaceList;
            use geo_types::Geometry as GtGeometry;
            use pyo3::prelude::*;

            /// Exchanges vector geometries between Rust and Python using [pyo3](https://pyo3.rs) and [Pythons `__geo_interface__` protocol](https://gist.github.com/sgillies/2217756).
            #[derive(Debug)]
            #[pyclass]
            pub struct Geometry(pub GtGeometry<$coord_type>);

            #[pymethods]
            impl Geometry {
                #[getter]
                fn __geo_interface__(&self, py: Python) -> PyResult<PyObject> {
                    self.0.as_geointerface_pyobject(py)
                }

                #[cfg(feature = "wkb")]
                #[getter]
                fn wkb<'py>(&self, py: Python<'py>) -> PyResult<&'py pyo3::types::PyBytes> {
                    use crate::wkb::WKBSupport;
                    let wkb_bytes = <$coord_type>::geometry_to_wkb(&self.0)?;
                    Ok(pyo3::types::PyBytes::new(py, &wkb_bytes))
                }
            }

            impl<'source> FromPyObject<'source> for Geometry {
                fn extract(ob: &'source PyAny) -> PyResult<Self> {
                    Ok(Self(ob.as_geometry()?))
                }
            }

            impl From<GtGeometry<$coord_type>> for Geometry {
                fn from(geom: GtGeometry<$coord_type>) -> Self {
                    Self(geom)
                }
            }

            macro_rules! geometry_enum_from_impl {
                ($geom_type:ty, $enum_variant_name:ident) => {
                    impl From<$geom_type> for Geometry {
                        fn from(g: $geom_type) -> Self {
                            Geometry(GtGeometry::$enum_variant_name(g))
                        }
                    }
                };
            }
            geometry_enum_from_impl!(geo_types::Point<$coord_type>, Point);
            geometry_enum_from_impl!(geo_types::MultiPoint<$coord_type>, MultiPoint);
            geometry_enum_from_impl!(geo_types::LineString<$coord_type>, LineString);
            geometry_enum_from_impl!(geo_types::MultiLineString<$coord_type>, MultiLineString);
            geometry_enum_from_impl!(geo_types::Polygon<$coord_type>, Polygon);
            geometry_enum_from_impl!(geo_types::MultiPolygon<$coord_type>, MultiPolygon);
            geometry_enum_from_impl!(
                geo_types::GeometryCollection<$coord_type>,
                GeometryCollection
            );
            geometry_enum_from_impl!(geo_types::Rect<$coord_type>, Rect);
            geometry_enum_from_impl!(geo_types::Line<$coord_type>, Line);
            geometry_enum_from_impl!(geo_types::Triangle<$coord_type>, Triangle);

            impl From<Geometry> for GtGeometry<$coord_type> {
                fn from(gw: Geometry) -> Self {
                    gw.0
                }
            }

            /// Vec of geometries
            ///
            /// Accessible from python via `__geo_interface__` as a list of geometries.
            #[derive(Debug)]
            #[pyclass]
            pub struct GeometryVec(pub Vec<GtGeometry<$coord_type>>);

            #[pymethods]
            impl GeometryVec {
                #[getter]
                fn __geo_interface__(&self, py: Python) -> PyResult<PyObject> {
                    self.0.as_geointerface_list_pyobject(py)
                }
            }

            impl<'source> FromPyObject<'source> for GeometryVec {
                fn extract(ob: &'source PyAny) -> PyResult<Self> {
                    Ok(ob.as_geometry_vec()?)
                }
            }

            impl From<GeometryVec> for Vec<GtGeometry<$coord_type>> {
                fn from(gv: GeometryVec) -> Self {
                    gv.0
                }
            }

            impl From<Vec<GtGeometry<$coord_type>>> for GeometryVec {
                fn from(geoms: Vec<GtGeometry<$coord_type>>) -> Self {
                    Self(geoms)
                }
            }

            pub trait AsGeometryVec {
                /// Creates a `GeometryVec` from `self`
                fn as_geometry_vec(&self) -> PyResult<GeometryVec>;
            }

            impl AsGeometryVec for PyAny {
                fn as_geometry_vec(&self) -> PyResult<GeometryVec> {
                    GeometryVec::extract(self)
                }
            }

            /// Vec of geometries
            ///
            /// Accessible from python via `__geo_interface__` as a FeatureCollection.
            #[derive(Debug)]
            #[pyclass]
            pub struct GeometryVecFc(pub Vec<GtGeometry<$coord_type>>);

            #[pymethods]
            impl GeometryVecFc {
                #[getter]
                fn __geo_interface__(&self, py: Python) -> PyResult<PyObject> {
                    self.0.as_geointerface_featurecollection_pyobject(py)
                }
            }

            impl<'source> FromPyObject<'source> for GeometryVecFc {
                fn extract(ob: &'source PyAny) -> PyResult<Self> {
                    Ok(ob.as_geometry_vec_fc()?)
                }
            }

            impl From<GeometryVecFc> for Vec<GtGeometry<$coord_type>> {
                fn from(gv: GeometryVecFc) -> Self {
                    gv.0
                }
            }

            impl From<Vec<GtGeometry<$coord_type>>> for GeometryVecFc {
                fn from(geoms: Vec<GtGeometry<$coord_type>>) -> Self {
                    Self(geoms)
                }
            }

            pub trait AsGeometryVecFc {
                /// Creates a `GeometryVecFc` from `self`
                fn as_geometry_vec_fc(&self) -> PyResult<GeometryVecFc>;
            }

            impl AsGeometryVecFc for PyAny {
                fn as_geometry_vec_fc(&self) -> PyResult<GeometryVecFc> {
                    GeometryVecFc::extract(self)
                }
            }
        }
    };
}

#[cfg(feature = "f64")]
dt_mod!(f64, f64);
#[cfg(feature = "f32")]
dt_mod!(f32, f32);
#[cfg(feature = "i8")]
dt_mod!(i8, i8);
#[cfg(feature = "i16")]
dt_mod!(i16, i16);
#[cfg(feature = "i32")]
dt_mod!(i32, i32);
#[cfg(feature = "i64")]
dt_mod!(i64, i64);
#[cfg(feature = "u8")]
dt_mod!(u8, u8);
#[cfg(feature = "u16")]
dt_mod!(u16, u16);
#[cfg(feature = "u32")]
dt_mod!(u32, u32);
#[cfg(feature = "u64")]
dt_mod!(u64, u64);
