# Changes

## Unreleased

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
