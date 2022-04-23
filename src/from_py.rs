use geo_types::{
    Coordinate, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint,
    MultiPolygon, Point, Polygon,
};
use pyo3::exceptions::PyValueError;
use pyo3::types::{PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};
use pyo3::{PyAny, PyErr, PyResult};

pub trait AsCoordinate {
    fn as_coordinate(&self) -> PyResult<Coordinate<f64>>;
}

impl AsCoordinate for [&PyAny] {
    fn as_coordinate(&self) -> PyResult<Coordinate<f64>> {
        check_length(self, 2)?;
        Ok((extract_as_float(self[0])?, extract_as_float(self[1])?).into())
    }
}

#[inline]
fn extract_as_float(obj: &PyAny) -> PyResult<f64> {
    if obj.is_instance_of::<PyFloat>()? {
        obj.downcast::<PyFloat>()?.extract::<f64>()
    } else if obj.is_instance_of::<PyInt>()? {
        Ok(obj.downcast::<PyInt>()?.extract::<i64>()? as f64)
    } else {
        Err(PyValueError::new_err(
            "coordinate values must be either float or int",
        ))
    }
}

#[inline]
fn tuple_map<'a, T, F>(obj: &'a PyAny, map_fn: F) -> PyResult<T>
where
    F: Fn(&'a PyTuple) -> PyResult<T>,
{
    if obj.is_instance_of::<PyTuple>()? {
        map_fn(obj.downcast::<PyTuple>()?)
    } else if obj.is_instance_of::<PyList>()? {
        map_fn(obj.downcast::<PyList>()?.as_sequence().tuple()?)
    } else {
        Err(PyValueError::new_err("expected either tuple or list"))
    }
}

impl AsCoordinate for PyAny {
    fn as_coordinate(&self) -> PyResult<Coordinate<f64>> {
        tuple_map(self, |tuple| tuple.as_coordinate())
    }
}

impl AsCoordinate for PyTuple {
    fn as_coordinate(&self) -> PyResult<Coordinate<f64>> {
        self.as_slice().as_coordinate()
    }
}

impl AsCoordinate for PyList {
    fn as_coordinate(&self) -> PyResult<Coordinate<f64>> {
        self.as_sequence().tuple()?.as_slice().as_coordinate()
    }
}

pub trait AsCoordinateVec {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<f64>>>;
}

impl AsCoordinateVec for [&PyAny] {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<f64>>> {
        self.iter()
            .map(|obj| obj.as_coordinate())
            .collect::<PyResult<Vec<_>>>()
    }
}

impl AsCoordinateVec for PyTuple {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<f64>>> {
        self.as_slice().as_coordinate_vec()
    }
}

impl AsCoordinateVec for PyList {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<f64>>> {
        self.as_sequence().tuple()?.as_coordinate_vec()
    }
}

impl AsCoordinateVec for PyAny {
    fn as_coordinate_vec(&self) -> PyResult<Vec<Coordinate<f64>>> {
        tuple_map(self, |tuple| tuple.as_coordinate_vec())
    }
}

fn check_length<T>(slice: &[T], required_length: usize) -> PyResult<()> {
    if slice.len() != required_length {
        Err(PyValueError::new_err(format!(
            "Expected length of {}, found {}",
            required_length,
            slice.len()
        )))
    } else {
        Ok(())
    }
}

pub trait AsGeometry {
    fn as_geometry(&self) -> PyResult<Geometry<f64>>;
}

impl AsGeometry for PyDict {
    fn as_geometry(&self) -> PyResult<Geometry<f64>> {
        extract_geometry(self, 0)
    }
}

fn extract_geometry(dict: &PyDict, level: u8) -> PyResult<Geometry<f64>> {
    if level > 1 {
        Err(PyValueError::new_err("recursion level exceeded"))
    } else {
        let geom_type = extract_geom_dict_value(dict, "type")?
            .downcast::<PyString>()?
            .extract::<String>()?;
        let coordinates = || extract_geom_dict_value(dict, "coordinates");
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
                        .map(|t| extract_polygon(t))
                        .collect::<PyResult<Vec<_>>>()
                },
            )?))),
            "GeometryCollection" => {
                let geoms = tuple_map(extract_geom_dict_value(dict, "geometries")?, |tuple| {
                    tuple
                        .as_slice()
                        .iter()
                        .map(|obj| {
                            obj.downcast::<PyDict>()
                                .map_err(PyErr::from)
                                .and_then(|obj_dict| extract_geometry(obj_dict, level + 1))
                        })
                        .collect::<Result<Vec<_>, _>>()
                })?;
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

fn extract_linestrings(obj: &PyAny) -> PyResult<Vec<LineString<f64>>> {
    tuple_map(obj, |tuple| {
        tuple
            .iter()
            .map(|t| tuple_map(t, |t| t.as_coordinate_vec().map(LineString::new)))
            .collect::<PyResult<Vec<_>>>()
    })
}

fn extract_polygon(obj: &PyAny) -> PyResult<Polygon<f64>> {
    let mut linestings = extract_linestrings(obj)?;
    if linestings.len() == 0 {
        return Err(PyValueError::new_err("Polygons require at least one ring"));
    }
    let exterior = linestings.remove(0);
    Ok(Polygon::new(exterior, linestings))
}

fn extract_geom_dict_value<'a>(dict: &'a PyDict, key: &str) -> PyResult<&'a PyAny> {
    if let Some(value) = dict.get_item(key) {
        Ok(value)
    } else {
        Err(PyValueError::new_err(format!(
            "geometry has \"{}\" not set",
            key
        )))
    }
}

impl AsGeometry for PyAny {
    fn as_geometry(&self) -> PyResult<Geometry<f64>> {
        // search for and call __geo_interface__ if its present
        if let Ok(geo_interface) = self.getattr("__geo_interface__") {
            if geo_interface.is_callable() {
                geo_interface.call0()?
            } else {
                geo_interface
            }
            .downcast::<PyDict>()?
            .as_geometry()
        } else {
            // fallback to attempt to access as dict
            self.downcast::<PyDict>()?.as_geometry()
        }
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
    use pyo3::{prepare_freethreaded_python, PyResult, Python};

    #[test]
    fn coordinate_from_pytuple() {
        prepare_freethreaded_python();
        Python::with_gil(|py| {
            let tuple = py.eval("(1.0, 2.0)", None, None).unwrap();
            let c = tuple.as_coordinate().unwrap();
            assert_eq!(c.x, 1.0);
            assert_eq!(c.y, 2.0);
        });
    }

    #[test]
    fn coordinate_from_pytuple_with_ints() {
        prepare_freethreaded_python();
        Python::with_gil(|py| {
            let tuple = py.eval("(1, 2)", None, None).unwrap();
            let c = tuple.as_coordinate().unwrap();
            assert_eq!(c.x, 1.0);
            assert_eq!(c.y, 2.0);
        });
    }

    #[test]
    fn coordinate_from_pylist() {
        prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = py.eval("[1.0, 2.0]", None, None).unwrap();
            let c = list.as_coordinate().unwrap();
            assert_eq!(c.x, 1.0);
            assert_eq!(c.y, 2.0);
        });
    }

    #[test]
    fn coordinate_sequence_from_pylist() {
        prepare_freethreaded_python();
        Python::with_gil(|py| {
            let list = py.eval("[[1.0, 2.0], (3.0, 4.)]", None, None).unwrap();
            let coords = list.as_coordinate_vec().unwrap();
            assert_eq!(coords.len(), 2);
            assert_eq!(coords[0].x, 1.0);
            assert_eq!(coords[0].y, 2.0);
            assert_eq!(coords[1].x, 3.0);
            assert_eq!(coords[1].y, 4.0);
        });
    }

    fn parse_geojson_geometry(geojson_str: &str) -> PyResult<Geometry<f64>> {
        prepare_freethreaded_python();
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
        prepare_freethreaded_python();
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
