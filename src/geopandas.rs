use crate::wrappers::f64::GeometryInterface;
use pyo3::prelude::*;

#[derive(Debug)]
#[pyclass]
pub struct GeoSeries(pub Vec<GeometryInterface>);

impl<'source> FromPyObject<'source> for GeoSeries {
    fn extract(ob: &'source PyAny) -> PyResult<Self> {
        let mut geoms = vec![];
        for item in ob.iter()? {
            let item = item?;
            geoms.push(GeometryInterface::extract(item)?);
        }
        geoms.shrink_to_fit();
        Ok(GeoSeries(geoms))
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
