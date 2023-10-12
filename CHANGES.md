# Changes

## Unreleased

## 0.7.0 - 2023-10-12
* Upgrade `pyo3` from 0.19 to 0.20.

## 0.6.2 - 2023-10-03
* Upgrade `geozero` from 0.9 to 0.11.

## 0.6.1 - 2023-07-24
* Upgrade `geozero` from 0.9 to 0.10.
* Do not access inner members of `geo_types` types. Only use the provided methods.

## 0.6.0 - 2023-06-10
* Upgrade `pyo3` from 0.18 to 0.19.

## 0.5.0 - 2023-01-19
* Upgrade `pyo3` from 0.17 to 0.18.

## 0.4.1 - 2022-11-30
* Fix deprecation warnings of `geo_types::Coordinate` by migrating to `geo_types::Coord`.

## 0.4.0

* Rename `GeometryInterface` struct to `Geometry` - that is the last rename of this struct - promised.
* Added support to exchange `Vec<Geometry>` with python. [#5](https://github.com/nmandery/py_geo_interface/pull/5)
* Upgrade `pyo3` from 0.16 to 0.17. [#7](https://github.com/nmandery/py_geo_interface/pull/7)

## 0.3.0

* Support exchanging geometries using Well-Known-Binary format. The `wkb`-property of `shapely`
  geometries will be used. Additionally, the `GeometryInterface`-type exposed to python will have a `wkb`-property
  itself. This is only supported for the `f64` variant of the `GeoInterface`.
* Rename `GeoInterface` struct to `GeometryInterface` to distinguish the provided geometry support from geo_interface features and featurecollections.
* Simplify lifetimes and rename `AsGeoInterfacePyDict` to `AsGeoInterface`.

## 0.2.0

* Generic `GeoInterface` implementations for `f64`, `f32`, `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32` and `i64`.

## 0.1.1

* Support for `Py_LIMTED`-API by avoiding usage of `PyTuple::as_slice()`

## 0.1.0

Initial release
