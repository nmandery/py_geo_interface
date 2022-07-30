use crate::PyCoordNum;
use geo_types::{
    Coordinate, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon,
};
use num_traits::NumCast;
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyFloat, PyInt, PyIterator, PyList, PyString, PyTuple};
use pyo3::{intern, PyAny, PyErr, PyResult};
use std::any::type_name;

pub trait AsCoordinate<T: PyCoordNum> {
    /// Creates a `Coordinate<T>` from `self`.
    fn as_coordinate(&self) -> PyResult<Coordinate<T>>;
}

pub trait ExtractFromPyFloat {
    fn extract_from_pyfloat(pf: &PyFloat) -> PyResult<Self>
    where
        Self: Sized;
}

macro_rules! extract_from_pyfloat_float {
    ($ftype:ty) => {
        impl ExtractFromPyFloat for $ftype {
            fn extract_from_pyfloat(pf: &PyFloat) -> PyResult<Self> {
                pf.extract::<Self>()
            }
        }
    };
}
extract_from_pyfloat_float!(f32);
extract_from_pyfloat_float!(f64);

macro_rules! extract_from_pyfloat_int {
    ($ftype:ty) => {
        impl ExtractFromPyFloat for $ftype {
            fn extract_from_pyfloat(pf: &PyFloat) -> PyResult<Self> {
                <Self as NumCast>::from(pf.extract::<f64>()?).ok_or_else(|| {
                    PyValueError::new_err(format!(
                        "Coordinate value can not be represented in {}",
                        type_name::<Self>()
                    ))
                })
            }
        }
    };
}
extract_from_pyfloat_int!(i8);
extract_from_pyfloat_int!(i16);
extract_from_pyfloat_int!(i32);
extract_from_pyfloat_int!(i64);
extract_from_pyfloat_int!(u8);
extract_from_pyfloat_int!(u16);
extract_from_pyfloat_int!(u32);
extract_from_pyfloat_int!(u64);

pub trait ExtractFromPyInt {
    fn extract_from_pyint(pf: &PyInt) -> PyResult<Self>
    where
        Self: Sized;
}

macro_rules! extract_from_pyint_float {
    ($ftype:ty) => {
        impl ExtractFromPyInt for $ftype {
            fn extract_from_pyint(pf: &PyInt) -> PyResult<Self> {
                <Self as NumCast>::from(pf.extract::<i64>()?).ok_or_else(|| {
                    PyValueError::new_err(format!(
                        "Coordinate value can not be represented in {}",
                        type_name::<Self>()
                    ))
                })
            }
        }
    };
}
extract_from_pyint_float!(f32);
extract_from_pyint_float!(f64);

macro_rules! extract_from_pyint_int {
    ($ftype:ty) => {
        impl ExtractFromPyInt for $ftype {
            fn extract_from_pyint(pf: &PyInt) -> PyResult<Self> {
                pf.extract::<Self>()
            }
        }
    };
}
extract_from_pyint_int!(i8);
extract_from_pyint_int!(i16);
extract_from_pyint_int!(i32);
extract_from_pyint_int!(i64);
extract_from_pyint_int!(u8);
extract_from_pyint_int!(u16);
extract_from_pyint_int!(u32);
extract_from_pyint_int!(u64);

#[inline]
fn extract_pycoordnum<T: PyCoordNum>(obj: &PyAny) -> PyResult<T> {
    if obj.is_instance_of::<PyFloat>()? {
        T::extract_from_pyfloat(obj.downcast::<PyFloat>()?)
    } else if obj.is_instance_of::<PyInt>()? {
        T::extract_from_pyint(obj.downcast::<PyInt>()?)
    } else {
        Err(PyValueError::new_err(
            "coordinate values must be either float or int",
        ))
    }
}

#[inline]
fn tuple_map<'a, O, F>(obj: &'a PyAny, map_fn: F) -> PyResult<O>
where
    F: Fn(&'a PyTuple) -> PyResult<O>,
{
    if obj.is_instance_of::<PyTuple>()? {
        map_fn(obj.downcast::<PyTuple>()?)
    } else if obj.is_instance_of::<PyList>()? {
        map_fn(obj.downcast::<PyList>()?.as_sequence().tuple()?)
    } else {
        Err(PyValueError::new_err("expected either tuple or list"))
    }
}

impl<T: PyCoordNum> AsCoordinate<T> for PyAny {
    fn as_coordinate(&self) -> PyResult<Coordinate<T>> {
        tuple_map(self, |tuple| tuple.as_coordinate())
    }
}

impl<T: PyCoordNum> AsCoordinate<T> for PyTuple {
    fn as_coordinate(&self) -> PyResult<Coordinate<T>> {
        if self.len() != 2 {
            return Err(PyValueError::new_err(format!(
                "Expected length of 2 values for coordinate, found {}",
                self.len()
            )));
        }
        let mut tuple_iter = self.iter();
        let x = extract_pycoordnum(tuple_iter.next().unwrap())?;
        let y = extract_pycoordnum(tuple_iter.next().unwrap())?;
        Ok((x, y).into())
    }
}

impl<T: PyCoordNum> AsCoordinate<T> for PyList {
    fn as_coordinate(&self) -> PyResult<Coordinate<T>> {
        self.as_sequence().tuple()?.as_coordinate()
    }
}

pub trait AsCoordinateVec<T: PyCoordNum> {
    /// Creates a `Vec<Coordinate<T>>` from `self`.
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<T>>>;
}

impl<T: PyCoordNum> AsCoordinateVec<T> for PyTuple {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<T>>> {
        self.iter().map(|tuple| tuple.as_coordinate()).collect()
    }
}

impl<T: PyCoordNum> AsCoordinateVec<T> for PyList {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<T>>> {
        self.as_sequence().tuple()?.as_coordinate_vec()
    }
}

impl<T: PyCoordNum> AsCoordinateVec<T> for PyAny {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<T>>> {
        tuple_map(self, |tuple| tuple.as_coordinate_vec())
    }
}

pub trait AsGeometry<T: PyCoordNum> {
    /// Creates a `Geometry<T>` from `self`
    fn as_geometry(&self) -> PyResult<Geometry<T>>;
}

impl<T: PyCoordNum> AsGeometry<T> for PyDict {
    fn as_geometry(&self) -> PyResult<Geometry<T>> {
        extract_geometry(self, 0)
    }
}

pub trait AsGeometryVec<T: PyCoordNum> {
    /// Creates a `Vec<Geometry<T>` from `self`
    fn as_geometry_vec(&self) -> PyResult<Vec<Geometry<T>>>;
}

impl<T: PyCoordNum> AsGeometryVec<T> for PyIterator {
    fn as_geometry_vec(&self) -> PyResult<Vec<Geometry<T>>> {
        let mut outvec = Vec::with_capacity(self.len().unwrap_or(0));
        for maybe_geom in self {
            outvec.push(maybe_geom?.as_geometry()?);
        }
        outvec.shrink_to_fit();
        Ok(outvec)
    }
}

impl<T: PyCoordNum> AsGeometryVec<T> for PyAny {
    fn as_geometry_vec(&self) -> PyResult<Vec<Geometry<T>>> {
        self.iter()?.as_geometry_vec()
    }
}

impl<T: PyCoordNum> AsGeometryVec<T> for PyList {
    fn as_geometry_vec(&self) -> PyResult<Vec<Geometry<T>>> {
        let mut outvec = Vec::with_capacity(self.len());
        for maybe_geom in self {
            outvec.push(maybe_geom.as_geometry()?);
        }
        Ok(outvec)
    }
}

fn extract_geometry<T: PyCoordNum>(dict: &PyDict, level: u8) -> PyResult<Geometry<T>> {
    if level > 1 {
        Err(PyValueError::new_err("recursion level exceeded"))
    } else {
        let geom_type = extract_geom_dict_value(dict, intern!(dict.py(), "type"))?
            .downcast::<PyString>()?
            .extract::<String>()?;
        let coordinates = || extract_geom_dict_value(dict, intern!(dict.py(), "coordinates"));
        match geom_type.as_str() {
            "Point" => Ok(Geometry::from(Point::from(coordinates()?.as_coordinate()?))),
            "MultiPoint" => Ok(Geometry::from(MultiPoint::from(
                coordinates()?
                    .as_coordinate_vec()?
                    .drain(..)
                    .map(Point::from)
                    .collect::<Vec<_>>(),
            ))),
            "LineString" => Ok(Geometry::from(LineString::from(
                coordinates()?.as_coordinate_vec()?,
            ))),
            "MultiLineString" => Ok(Geometry::from(MultiLineString::new(extract_linestrings(
                coordinates()?,
            )?))),
            "Polygon" => Ok(Geometry::from(extract_polygon(coordinates()?)?)),
            "MultiPolygon" => Ok(Geometry::from(MultiPolygon::new(tuple_map(
                coordinates()?,
                |tuple| {
                    tuple
                        .iter()
                        .map(extract_polygon)
                        .collect::<PyResult<Vec<_>>>()
                },
            )?))),
            "GeometryCollection" => {
                let geoms = tuple_map(
                    extract_geom_dict_value(dict, intern!(dict.py(), "geometries"))?,
                    |tuple| {
                        tuple
                            .iter()
                            .map(|obj| {
                                obj.downcast::<PyDict>()
                                    .map_err(PyErr::from)
                                    .and_then(|obj_dict| extract_geometry(obj_dict, level + 1))
                            })
                            .collect::<Result<Vec<_>, _>>()
                    },
                )?;
                Ok(Geometry::GeometryCollection(GeometryCollection::new_from(
                    geoms,
                )))
            }
            _ => Err(PyValueError::new_err(format!(
                "Unsupported geometry type \"{}\"",
                geom_type
            ))),
        }
    }
}

fn extract_linestrings<T: PyCoordNum>(obj: &PyAny) -> PyResult<Vec<LineString<T>>> {
    tuple_map(obj, |tuple| {
        tuple
            .iter()
            .map(|t| tuple_map(t, |t| t.as_coordinate_vec().map(LineString::new)))
            .collect::<PyResult<Vec<_>>>()
    })
}

fn extract_polygon<T: PyCoordNum>(obj: &PyAny) -> PyResult<Polygon<T>> {
    let mut linestings = extract_linestrings(obj)?;
    if linestings.is_empty() {
        return Err(PyValueError::new_err("Polygons require at least one ring"));
    }
    let exterior = linestings.remove(0);
    Ok(Polygon::new(exterior, linestings))
}

fn extract_geom_dict_value<'a>(dict: &'a PyDict, key: &PyString) -> PyResult<&'a PyAny> {
    if let Some(value) = dict.get_item(key) {
        Ok(value)
    } else {
        Err(PyValueError::new_err(format!(
            "geometry has \"{}\" not set",
            key
        )))
    }
}

impl<T: PyCoordNum> AsGeometry<T> for PyAny {
    fn as_geometry(&self) -> PyResult<Geometry<T>> {
        #[cfg(feature = "wkb")]
        if let Some(geom) = T::read_wkb_property(self)? {
            return Ok(geom);
        }

        if let Some(geom) = read_geointerface(self)? {
            Ok(geom)
        } else {
            // fallback and attempt to access as dict
            self.downcast::<PyDict>()?.as_geometry()
        }
    }
}

/// search for and call __geo_interface__ if its present
fn read_geointerface<T: PyCoordNum>(value: &PyAny) -> PyResult<Option<Geometry<T>>> {
    if let Ok(geo_interface) = value.getattr(intern!(value.py(), "__geo_interface__")) {
        let geom = if geo_interface.is_callable() {
            geo_interface.call0()?
        } else {
            geo_interface
        }
        .downcast::<PyDict>()?
        .as_geometry()?;
        Ok(Some(geom))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    //! most data used in these testcases is from the GeoJSON RFC
    //! https://datatracker.ietf.org/doc/html/rfc7946
    //!
    use crate::from_py::{AsCoordinate, AsCoordinateVec, AsGeometry};
    use geo_types::{
        Coordinate, Geometry, GeometryCollection, LineString, MultiPoint, MultiPolygon, Point,
        Polygon,
    };
    use pyo3::types::{PyDict, PyString};
    use pyo3::{PyResult, Python};

    #[test]
    fn coordinate_from_pytuple() {
        Python::with_gil(|py| {
            let tuple = py.eval("(1.0, 2.0)", None, None).unwrap();
            let c: Coordinate<f64> = tuple.as_coordinate().unwrap();
            assert_eq!(c.x, 1.0);
            assert_eq!(c.y, 2.0);
        });
    }

    #[test]
    fn coordinate_from_pytuple_cast_ints() {
        Python::with_gil(|py| {
            let tuple = py.eval("(1, 2)", None, None).unwrap();
            let c: Coordinate<f64> = tuple.as_coordinate().unwrap();
            assert_eq!(c.x, 1.0);
            assert_eq!(c.y, 2.0);
        });
    }

    #[test]
    fn coordinate_from_pytuple_to_ints() {
        Python::with_gil(|py| {
            let tuple = py.eval("(1, 2)", None, None).unwrap();
            let c: Coordinate<i32> = tuple.as_coordinate().unwrap();
            assert_eq!(c.x, 1);
            assert_eq!(c.y, 2);
        });
    }

    #[test]
    fn coordinate_from_pylist() {
        Python::with_gil(|py| {
            let list = py.eval("[1.0, 2.0]", None, None).unwrap();
            let c: Coordinate<f64> = list.as_coordinate().unwrap();
            assert_eq!(c.x, 1.0);
            assert_eq!(c.y, 2.0);
        });
    }

    #[test]
    fn coordinate_sequence_from_pylist() {
        Python::with_gil(|py| {
            let list = py.eval("[[1.0, 2.0], (3.0, 4.)]", None, None).unwrap();
            let coords: Vec<Coordinate<f64>> = list.as_coordinate_vec().unwrap();
            assert_eq!(coords.len(), 2);
            assert_eq!(coords[0].x, 1.0);
            assert_eq!(coords[0].y, 2.0);
            assert_eq!(coords[1].x, 3.0);
            assert_eq!(coords[1].y, 4.0);
        });
    }

    fn parse_geojson_geometry(geojson_str: &str) -> PyResult<Geometry<f64>> {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            locals.set_item("gj", PyString::new(py, geojson_str))?;
            py.run(r#"import json"#, None, Some(locals))?;
            py.eval(r#"json.loads(gj)"#, None, Some(locals))?
                .as_geometry()
        })
    }

    #[test]
    fn read_point() {
        let geom = parse_geojson_geometry(
            r#"
{
    "type": "Point",
    "coordinates": [100.0, 5.0]
}
            "#,
        )
        .unwrap();
        assert_eq!(geom, Geometry::Point(Point::new(100., 5.)));
    }

    #[test]
    fn read_multipoint() {
        let geom = parse_geojson_geometry(
            r#"
{
    "type": "MultiPoint",
    "coordinates": [
        [100.0, 0.0],
        [101.0, 1.0]
    ]
}             
            "#,
        )
        .unwrap();
        assert_eq!(
            geom,
            Geometry::MultiPoint(MultiPoint::from(vec![
                Point::from(Coordinate::from((100., 0.))),
                Point::from(Coordinate::from((101., 1.)))
            ]))
        );
    }

    #[test]
    fn read_linestring() {
        let geom = parse_geojson_geometry(
            r#"
{
    "type": "LineString",
    "coordinates": [
        [100.0, 0.0],
        [101.0, 1.0]
    ]
}             
            "#,
        )
        .unwrap();
        assert_eq!(
            geom,
            Geometry::LineString(LineString::from(vec![
                Coordinate::from((100., 0.)),
                Coordinate::from((101., 1.))
            ]))
        );
    }

    #[test]
    fn read_polygon() {
        let geom = parse_geojson_geometry(
            r#"
{
    "type": "Polygon",
    "coordinates": [
        [
            [100.0, 0.0],
            [101.0, 0.0],
            [101.0, 1.0],
            [100.0, 1.0],
            [100.0, 0.0]
        ],
        [
            [100.8, 0.8],
            [100.8, 0.2],
            [100.2, 0.2],
            [100.2, 0.8],
            [100.8, 0.8]
        ]
    ]
}
            "#,
        )
        .unwrap();
        assert_eq!(
            geom,
            Geometry::Polygon(Polygon::new(
                LineString::from(vec![
                    Coordinate::from((100., 0.)),
                    Coordinate::from((101., 0.)),
                    Coordinate::from((101., 1.)),
                    Coordinate::from((100., 1.)),
                    Coordinate::from((100., 0.)),
                ]),
                vec![LineString::from(vec![
                    Coordinate::from((100.8, 0.8)),
                    Coordinate::from((100.8, 0.2)),
                    Coordinate::from((100.2, 0.2)),
                    Coordinate::from((100.2, 0.8)),
                    Coordinate::from((100.8, 0.8)),
                ])]
            ))
        );
    }

    #[test]
    fn read_multipolygon() {
        let geom = parse_geojson_geometry(
            r#"
{
    "type": "MultiPolygon",
    "coordinates": [
        [
            [
                [102.0, 2.0],
                [103.0, 2.0],
                [103.0, 3.0],
                [102.0, 3.0],
                [102.0, 2.0]
            ]
        ],
        [
            [
                [100.0, 0.0],
                [101.0, 0.0],
                [101.0, 1.0],
                [100.0, 1.0],
                [100.0, 0.0]
            ],
            [
                [100.2, 0.2],
                [100.2, 0.8],
                [100.8, 0.8],
                [100.8, 0.2],
                [100.2, 0.2]
            ]
        ]
    ]
}     
            "#,
        )
        .unwrap();
        assert_eq!(
            geom,
            Geometry::MultiPolygon(MultiPolygon::new(vec![
                Polygon::new(
                    LineString::from(vec![
                        Coordinate::from((102., 2.)),
                        Coordinate::from((103., 2.)),
                        Coordinate::from((103., 3.)),
                        Coordinate::from((102., 3.)),
                        Coordinate::from((102., 2.)),
                    ]),
                    vec![]
                ),
                Polygon::new(
                    LineString::from(vec![
                        Coordinate::from((100., 0.)),
                        Coordinate::from((101., 0.)),
                        Coordinate::from((101., 1.)),
                        Coordinate::from((100., 1.)),
                        Coordinate::from((100., 0.)),
                    ]),
                    vec![LineString::from(vec![
                        Coordinate::from((100.2, 0.2)),
                        Coordinate::from((100.2, 0.8)),
                        Coordinate::from((100.8, 0.8)),
                        Coordinate::from((100.8, 0.2)),
                        Coordinate::from((100.2, 0.2)),
                    ])]
                )
            ]))
        );
    }

    #[test]
    fn read_geometrycollection() {
        let geom = parse_geojson_geometry(
            r#"
{
    "type": "GeometryCollection",
    "geometries": [{
        "type": "Point",
        "coordinates": [100.0, 0.0]
    }, {
        "type": "LineString",
        "coordinates": [
            [101.0, 0.0],
            [102.0, 1.0]
        ]
    }]
}          
            "#,
        )
        .unwrap();
        assert_eq!(
            geom,
            Geometry::GeometryCollection(GeometryCollection::new_from(vec![
                Geometry::Point(Point::from(Coordinate::from((100., 0.)))),
                Geometry::LineString(LineString::from(vec![
                    Coordinate::from((101., 0.)),
                    Coordinate::from((102., 1.))
                ]))
            ]))
        );
    }

    #[test]
    fn read_point_using_geointerface() {
        let geom = Python::with_gil(|py| {
            py.run(
                r#"
class Something:
    @property
    def __geo_interface__(self):
        return {"type": "Point", "coordinates": [5., 3.]}
            "#,
                None,
                None,
            )?;
            py.eval(r#"Something()"#, None, None)?.as_geometry()
        })
        .unwrap();
        assert_eq!(geom, Geometry::Point(Point::new(5., 3.)));
    }
}
