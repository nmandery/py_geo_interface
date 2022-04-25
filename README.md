# py_geo_interface

[![Latest Version](https://img.shields.io/crates/v/py_geo_interface.svg)](https://crates.io/crates/py_geo_interface) 
[![Documentation](https://docs.rs/py_geo_interface/badge.svg)](https://docs.rs/py_geo_interface)
![ci](https://github.com/nmandery/py_geo_interface/workflows/CI/badge.svg)
[![dependency status](https://deps.rs/repo/github/nmandery/py_geo_interface/status.svg)](https://deps.rs/repo/github/nmandery/h3ron)

Exchange vector geometries between Rust and Python using [pyo3](https://pyo3.rs) and [Pythons `__geo_interface__` protocol](https://gist.github.com/sgillies/2217756).

The `__geo_interface__` protocol is implemented by most popular geospatial python modules like `shapely`, `geojson`, `geopandas`, ....

For usage examples see the [documentation](https://docs.rs/py_geo_interface). 

[Changelog](./CHANGES.md)

### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>
