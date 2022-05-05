macro_rules! dt_mod {
    ($coord_type:ty, $mod_name:ident) => {
        pub mod $mod_name {
            use crate::from_py::AsGeometry;
            use crate::to_py::AsGeoInterfacePyDict;
            use pyo3::prelude::*;
            use pyo3::types::PyDict;

            /// Exchanges vector geometries between Rust and Python using [pyo3](https://pyo3.rs) and [Pythons `__geo_interface__` protocol](https://gist.github.com/sgillies/2217756).
            #[derive(Debug)]
            #[pyclass]
            pub struct GeoInterface(pub geo_types::Geometry<$coord_type>);

            #[pymethods]
            impl GeoInterface {
                #[getter]
                fn __geo_interface__<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
                    self.0.as_geointerface_pydict(py)
                }

                #[cfg(feature = "wkb")]
                #[getter]
                fn wkb<'py>(&self, py: Python<'py>) -> PyResult<&'py pyo3::types::PyBytes> {
                    use crate::wkb::WKBSupport;
                    let wkb_bytes = <$coord_type>::geometry_to_wkb(&self.0)?;
                    Ok(pyo3::types::PyBytes::new(py, &wkb_bytes))
                }
            }

            impl<'source> FromPyObject<'source> for GeoInterface {
                fn extract(ob: &'source PyAny) -> PyResult<Self> {
                    Ok(GeoInterface(ob.as_geometry()?))
                }
            }

            impl From<geo_types::Geometry<$coord_type>> for GeoInterface {
                fn from(geom: geo_types::Geometry<$coord_type>) -> Self {
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

            impl From<GeoInterface> for geo_types::Geometry<$coord_type> {
                fn from(gi: GeoInterface) -> Self {
                    gi.0
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
