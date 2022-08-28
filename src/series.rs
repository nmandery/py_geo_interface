use crate::from_py::AsGeometryVec;
use geo_types::Geometry;
use pyo3::prelude::*;

#[derive(Debug)]
#[pyclass]
pub struct GeometrySeries(pub Vec<Geometry<f64>>);

impl<'source> FromPyObject<'source> for GeometrySeries {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        Ok(GeometrySeries(ob.as_geometry_vec()?))
    }
}

impl From<GeometrySeries> for Vec<Geometry<f64>> {
    fn from(geoseries: GeometrySeries) -> Self {
        geoseries.0
    }
}

pub trait AsGeometrySeries {
    /// Creates a `GeometrySeries` from `self`
    fn as_geometry_series(&self) -> PyResult<GeometrySeries>;
}

impl AsGeometrySeries for PyAny {
    fn as_geometry_series(&self) -> PyResult<GeometrySeries> {
        GeometrySeries::extract(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::series::AsGeometrySeries;
    use pyo3::Python;

    #[test]
    fn geoseries_from_geopandas_geoseries() {
        let geoseries = Python::with_gil(|py| {
            py.run(
                r#"
import geopandas as gpd
world = gpd.read_file(gpd.datasets.get_path('naturalearth_lowres'))
            "#,
                None,
                None,
            )?;
            py.eval(r#"world.geometry"#, None, None)?
                .as_geometry_series()
        })
        .unwrap();
        assert!(geoseries.0.len() > 100);
    }
}
