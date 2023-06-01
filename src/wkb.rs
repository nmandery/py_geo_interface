use geo_types::{CoordNum, Geometry as GtGeometry};
use geozero::wkb::{FromWkb, WkbDialect, WkbWriter};
use geozero::{CoordDimensions, GeozeroGeometry};
use pyo3::exceptions::{PyNotImplementedError, PyValueError};
use pyo3::types::{PyByteArray, PyBytes};
use pyo3::{intern, PyAny, PyResult};
use std::io::Cursor;

pub trait WKBSupport {
    /// attempt to read the geometry from the objects `wkb` property if this exists.
    ///
    /// This supports reading from shapely geometries while skipping the geo_interface
    fn read_wkb_property(_value: &PyAny) -> PyResult<Option<GtGeometry<Self>>>
    where
        Self: CoordNum,
    {
        Ok(None)
    }

    fn geometry_to_wkb(_geom: &GtGeometry<Self>) -> PyResult<Vec<u8>>
    where
        Self: CoordNum,
    {
        Err(PyNotImplementedError::new_err(
            "Coordinate type can not be serialized to WKB",
        ))
    }
}

macro_rules! unsupported_wkb_conversion {
    ($coord_type:ty) => {
        impl WKBSupport for $coord_type {}
    };
}

unsupported_wkb_conversion!(u8);
unsupported_wkb_conversion!(u16);
unsupported_wkb_conversion!(u32);
unsupported_wkb_conversion!(u64);
unsupported_wkb_conversion!(i8);
unsupported_wkb_conversion!(i16);
unsupported_wkb_conversion!(i32);
unsupported_wkb_conversion!(i64);
unsupported_wkb_conversion!(f32);

impl WKBSupport for f64 {
    fn read_wkb_property(value: &PyAny) -> PyResult<Option<GtGeometry<Self>>> {
        if let Ok(wkb_attr) = value.getattr(intern!(value.py(), "wkb")) {
            let wkb = if wkb_attr.is_callable() {
                wkb_attr.call0()?
            } else {
                wkb_attr
            };
            let slice = if wkb.is_instance_of::<PyBytes>() {
                wkb.downcast::<PyBytes>()?.as_bytes()
            } else if wkb.is_instance_of::<PyByteArray>() {
                unsafe { wkb.downcast::<PyByteArray>()?.as_bytes() }
            } else {
                return Ok(None);
            };
            let mut cursor = Cursor::new(slice);

            let geom = GtGeometry::from_wkb(&mut cursor, WkbDialect::Wkb)
                .map_err(|e| PyValueError::new_err(format!("unable to parse WKB: {:?}", e)))?;
            Ok(Some(geom))
        } else {
            Ok(None)
        }
    }

    fn geometry_to_wkb(geom: &GtGeometry<Self>) -> PyResult<Vec<u8>>
    where
        Self: CoordNum,
    {
        let mut wkb: Vec<u8> = Vec::new();
        let mut writer = WkbWriter::new(&mut wkb, WkbDialect::Wkb);
        writer.dims = CoordDimensions::xy();
        geom.process_geom(&mut writer)
            .map_err(|e| PyValueError::new_err(format!("Unable to convert to WKB: {:?}", e)))?;
        Ok(wkb)
    }
}

#[cfg(all(test, feature = "f64"))]
mod tests {
    use crate::from_py::AsGeometry;
    use crate::Geometry;
    use geo_types::{Geometry as GtGeometry, Point};
    use pyo3::types::PyDict;
    use pyo3::{IntoPy, Python};

    #[test]
    fn geometry_from_shapely_wkb_bytes_property() {
        let geom = Python::with_gil(|py| {
            py.run(r#"from shapely.geometry import Point"#, None, None)?;
            py.eval(r#"Point(2.0, 4.0)"#, None, None)?.as_geometry()
        })
        .unwrap();
        assert_eq!(geom, GtGeometry::Point(Point::new(2., 4.)));
    }

    #[test]
    fn geometry_from_wkb_bytearray_property() {
        let geom = Python::with_gil(|py| {
            py.run(
                r#"
class Something:
    @property
    def wkb(self):
        return bytearray.fromhex("000000000140000000000000004010000000000000")
            "#,
                None,
                None,
            )?;
            py.eval(r#"Something()"#, None, None)?.as_geometry()
        })
        .unwrap();
        assert_eq!(geom, GtGeometry::Point(Point::new(2., 4.)));
    }

    #[test]
    fn geometryinterface_wkb_property() {
        Python::with_gil(|py| {
            let geom: Geometry = Point::new(2.0_f64, 4.0_f64).into();
            let locals = PyDict::new(py);
            locals.set_item("geom", geom.into_py(py)).unwrap();

            py.run(
                r#"
from shapely.geometry import Point
Point(2.0, 4.0).wkb == geom.wkb
"#,
                None,
                Some(locals),
            )
            .unwrap();
        });
    }
}
