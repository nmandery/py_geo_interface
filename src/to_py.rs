use crate::PyCoordNum;
use geo_types::{
    Coord, Geometry, GeometryCollection, Line, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon,
};
use pyo3::prelude::PyDictMethods;
use pyo3::types::{PyDict, PyList, PyTuple};
use pyo3::{intern, Bound, Py, PyAny, PyResult, Python, BoundObject};
use std::borrow::Borrow;
use std::iter::once;

/// Convert `self` to a Python dictionary reflecting the structure of a `__geo_interface__` python dict.
pub trait AsGeoInterface {
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;
}

impl<T> AsGeoInterface for Geometry<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self {
            Geometry::Point(g) => g.as_geointerface_pyobject(py),
            Geometry::Line(g) => g.as_geointerface_pyobject(py),
            Geometry::LineString(g) => g.as_geointerface_pyobject(py),
            Geometry::Polygon(g) => g.as_geointerface_pyobject(py),
            Geometry::MultiPoint(g) => g.as_geointerface_pyobject(py),
            Geometry::MultiLineString(g) => g.as_geointerface_pyobject(py),
            Geometry::MultiPolygon(g) => g.as_geointerface_pyobject(py),
            Geometry::GeometryCollection(g) => g.as_geointerface_pyobject(py),
            Geometry::Rect(g) => g.to_polygon().as_geointerface_pyobject(py),
            Geometry::Triangle(g) => g.to_polygon().as_geointerface_pyobject(py),
        }
    }
}

impl<T> AsGeoInterface for Point<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        make_geom_pyobject(py, intern!(py, "Point").clone().into_any(), Coord::from(*self).to_py(py)?)
    }
}

impl<T> AsGeoInterface for MultiPoint<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        make_geom_pyobject(
            py,
            intern!(py, "MultiPoint").clone().into_any(),
            coord_iter_to_py(self.iter().copied().map(Coord::from), py)?,
        )
    }
}

impl<T> AsGeoInterface for LineString<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        make_geom_pyobject(
            py,
            intern!(py, "LineString").clone().into_any(),
            coord_iter_to_py(self.coords(), py)?,
        )
    }
}

impl<T> AsGeoInterface for MultiLineString<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Remove vec allocation? Only used to have an ExactSizeIterator
        let linestrings: Vec<_> = self
            .iter()
            .map(|linestring| coord_iter_to_py(linestring.coords(), py))
            .collect::<PyResult<Vec<_>>>()?;

        make_geom_pyobject(
            py,
            intern!(py, "MultiLineString").clone().into_any(),
            PyTuple::new(py, linestrings)?.into_any(),
        )
    }
}

impl<T> AsGeoInterface for Line<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        make_geom_pyobject(
            py,
            intern!(py, "LineString").clone().into_any(),
            PyTuple::new(py, [self.start.to_py(py)?, self.end.to_py(py)?])?.into_any(),
        )
    }
}

fn polygon_coordinates_to_pyobject<'py, T>(
    py: Python<'py>,
    polygon: &Polygon<T>,
) -> PyResult<Bound<'py, PyAny>>
where
    T: PyCoordNum,
{
    let linestring_objs: Vec<_> = once(coord_iter_to_py(polygon.exterior().coords(), py))
        .chain(
            polygon
                .interiors()
                .iter()
                .map(|ls| coord_iter_to_py(ls.coords(), py)),
        )
        .collect::<PyResult<Vec<_>>>()?;
    Ok(PyTuple::new(py, linestring_objs)?.into_any())
}

impl<T> AsGeoInterface for Polygon<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        make_geom_pyobject(
            py,
            intern!(py, "Polygon").clone().into_any(),
            polygon_coordinates_to_pyobject(py, self)?,
        )
    }
}

impl<T> AsGeoInterface for MultiPolygon<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Remove vec allocation? Only used to have an ExactSizeIterator
        let polygons: Vec<_> = self
            .iter()
            .map(|polygon| polygon_coordinates_to_pyobject(py, polygon))
            .collect::<PyResult<Vec<_>>>()?;

        make_geom_pyobject(
            py,
            intern!(py, "MultiPolygon").clone().into_any(),
            PyTuple::new(py, polygons)?.into_any(),
        )
    }
}

fn make_geom_pyobject<'py>(
    py: Python<'py>,
    geom_type: Bound<'py, PyAny>,
    coordinates: Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item(intern!(py, "type"), geom_type)?;
    dict.set_item(intern!(py, "coordinates"), coordinates)?;
    Ok(dict.into_any())
}

impl<T> AsGeoInterface for GeometryCollection<T>
where
    T: PyCoordNum,
{
    fn as_geointerface_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let dict = PyDict::new(py);
        dict.set_item(intern!(py, "type"), intern!(py, "GeometryCollection"))?;

        // Remove vec allocation? Only used to have an ExactSizeIterator
        let geometries: Vec<_> = self
            .iter()
            .map(|geom| geom.as_geointerface_pyobject(py))
            .collect::<PyResult<Vec<_>>>()?;

        dict.set_item(
            intern!(py, "geometries"),
            PyTuple::new(py, geometries)?,
        )?;
        Ok(dict.into_any())
    }
}

fn coord_iter_to_py<'py, I, B, T>(
    iter: I,
    py: Python<'py>,
) -> PyResult<Bound<'py, PyAny>>
where
    I: Iterator<Item = B>,
    B: Borrow<Coord<T>>,
    T: PyCoordNum,
{
    // Remove vec allocation? Only used to have an ExactSizeIterator
    let elements: Vec<_> = iter
        .map(|coord| coord.borrow().to_py(py))
        .collect::<PyResult<Vec<_>>>()?;

    Ok(PyTuple::new(py, elements)?.into_any())
}

trait ToPy {
    fn to_py<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;
}

impl<T> ToPy for Coord<T>
where
    T: PyCoordNum,
{
    fn to_py<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Numeric types have infallible IntoPyObject, so .unwrap() is safe
        // Use .ok().unwrap() to avoid Debug bound on error type
        let x_obj: Py<PyAny> = self.x.into_pyobject(py).ok().unwrap().into_any().unbind();
        let y_obj: Py<PyAny> = self.y.into_pyobject(py).ok().unwrap().into_any().unbind();
        let items: Vec<Bound<'py, PyAny>> = vec![x_obj.into_bound(py), y_obj.into_bound(py)];
        Ok(PyTuple::new(py, items)?.into_any())
    }
}

impl<T> ToPy for [Coord<T>]
where
    T: PyCoordNum,
{
    fn to_py<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let elements: Vec<_> = self.iter().map(|c| c.to_py(py)).collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, elements)?.into_any())
    }
}

pub trait AsGeoInterfaceList {
    /// return self as a python list of `__geo_interface__`-representations of geometries
    fn as_geointerface_list_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;
}

impl<T> AsGeoInterfaceList for &[Geometry<T>]
where
    T: PyCoordNum,
{
    fn as_geointerface_list_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let geometries = self
            .iter()
            .map(|g| g.as_geointerface_pyobject(py))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyList::new(py, geometries)?.into_any())
    }
}

impl<T> AsGeoInterfaceList for Vec<Geometry<T>>
where
    T: PyCoordNum,
{
    fn as_geointerface_list_pyobject<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.as_slice().as_geointerface_list_pyobject(py)
    }
}

pub trait AsGeoInterfaceFeatureCollection {
    /// return self as a python `__geo_interface__` FeatureCollection
    fn as_geointerface_featurecollection_pyobject<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>>;
}

impl<T> AsGeoInterfaceFeatureCollection for &[Geometry<T>]
where
    T: PyCoordNum,
{
    fn as_geointerface_featurecollection_pyobject<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let featurecollection = PyDict::new(py);
        featurecollection.set_item(intern!(py, "type"), intern!(py, "FeatureCollection"))?;

        let features = self
            .iter()
            .map(|geom| geom_as_py_feature(py, geom))
            .collect::<PyResult<Vec<_>>>()?;

        featurecollection.set_item(intern!(py, "features"), features)?;
        Ok(featurecollection.into_any())
    }
}

impl<T> AsGeoInterfaceFeatureCollection for Vec<Geometry<T>>
where
    T: PyCoordNum,
{
    fn as_geointerface_featurecollection_pyobject<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Bound<'py, PyAny>> {
        self.as_slice()
            .as_geointerface_featurecollection_pyobject(py)
    }
}

fn geom_as_py_feature<'py, T>(
    py: Python<'py>,
    geom: &Geometry<T>,
) -> PyResult<Bound<'py, PyAny>>
where
    T: PyCoordNum,
{
    let feature = PyDict::new(py);
    feature.set_item(intern!(py, "type"), intern!(py, "Feature"))?;
    feature.set_item(intern!(py, "properties"), PyDict::new(py))?;
    feature.set_item(intern!(py, "geometry"), geom.as_geointerface_pyobject(py)?)?;
    Ok(feature.into_any())
}

#[cfg(all(test, feature = "f64"))]
mod tests {
    use crate::wrappers::f64::GeometryVecFc;
    use geo_types::{Geometry as GtGeometry, Point};
    use pyo3::prelude::PyDictMethods;
    use pyo3::types::PyDict;
    use pyo3::{Python};

    #[test]
    fn geopandas_from_features() {
        let geometries: GeometryVecFc = vec![
            GtGeometry::Point(Point::new(1.0f64, 3.0)),
            GtGeometry::Point(Point::new(2.0, 6.0)),
        ]
        .into();

        Python::attach(|py| {
            let locals = PyDict::new(py);
            locals
                .set_item("feature_collection", geometries)
                .unwrap();

            py.run(
                c"
import geopandas as gpd
from shapely.geometry import Point

gdf = gpd.GeoDataFrame.from_features(feature_collection)
assert len(gdf) == 2
assert gdf.geometry[0] == Point(1,3)
assert gdf.geometry[1] == Point(2,6)
            ",
                None,
                Some(&locals),
            )
            .unwrap();
        });
    }
}