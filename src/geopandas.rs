use crate::from_py::AsGeometryVec;
use geo_types::Geometry;
use pyo3::prelude::*;

#[derive(Debug)]
#[pyclass]
pub struct GeoSeries(pub Vec<Geometry<f64>>);

impl<'source> FromPyObject<'source> for GeoSeries {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(GeoSeries(ob.as_geometry_vec()?))
    }
}

impl From<GeoSeries> for Vec<Geometry<f64>> {
    fn from(geoseries: GeoSeries) -> Self {
        geoseries.0
    }
}

pub trait AsGeoSeries {
    /// Creates a `GeoSeries` from `self`
    fn as_geoseries(&self) -> PyResult<GeoSeries>;
}

impl AsGeoSeries for PyAny {
    fn as_geoseries(&self) -> PyResult<GeoSeries> {
        GeoSeries::extract(self)
    }
}

/*
#[pymethods]
impl GeoSeries {
    pub fn to_geopandas(&self) -> PyResult<PyObject> {
        todo!()
    }
}

 */

#[cfg(test)]
mod tests {
    use crate::geopandas::AsGeoSeries;
    use pyo3::{prepare_freethreaded_python, Python};

    #[test]
    fn geoseries_from_geopandas_geoseries() {
        prepare_freethreaded_python();
        let geoseries = Python::with_gil(|py| {
            py.run(
                r#"
import geopandas as gpd
world = gpd.read_file(gpd.datasets.get_path('naturalearth_lowres'))
            "#,
                None,
                None,
            )?;
            py.eval(r#"world.geometry"#, None, None)?.as_geoseries()
        })
        .unwrap();
        assert!(geoseries.0.len() > 100);
    }
}
