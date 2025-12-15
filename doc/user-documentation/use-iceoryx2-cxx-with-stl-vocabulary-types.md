# Using iceoryx2-cxx with STL vocabulary types

The iceoryx2 C++ bindings support the replacement of the home-grown `Optional`
and `Expected` with the `std::optional` and `std::expected`.

Since the iceoryx2 C++ bindings support C++14 on most platforms, the C++ version
also needs to be increased when the `std::optional` (C++14) and/or
`std::expected` (C++23) shall be used.

The CMake options are:
* `IOX2_CXX_STD_VERSION`
* `IOX2_BB_CXX_CONFIG_USE_STD_EXPECTED`
* `IOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL`

The following examples shows how to use both, the `std::optional` and
`std::expected` for the iceroyx2 C++ bindings:

```cmake
cmake -S . \
      -B target/ff/cc/build \
      -DIOX2_CXX_STD_VERSION=23 \
      -DIOX2_BB_CXX_CONFIG_USE_STD_EXPECTED=ON \
      -DIOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL=ON
cmake --build target/ff/cc/build
```
