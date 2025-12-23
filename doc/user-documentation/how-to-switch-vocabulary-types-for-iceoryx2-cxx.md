# How to switch vocabulary types for the iceoryx2 C++ bindings

The iceoryx2 C++ bindings support the replacement of the home-grown `Optional`
and `Expected` with the `std::optional` and `std::expected` or custom
implementations.

## Using iceoryx2-cxx with STL vocabulary types

Since the iceoryx2 C++ bindings use C++14 by default on most platforms, the C++
version also needs to be increased when the `std::optional` (C++17) and/or
`std::expected` (C++23) shall be used.

The relevant CMake options are:

* `IOX2_BB_CXX_CONFIG_USE_STD_EXPECTED`
* `IOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL`

The following examples shows how to use both, the `std::optional` and
`std::expected` for the iceroyx2 C++ bindings:

```cmake
cmake -S . \
      -B target/ff/cc/build \
      -DIOX2_BB_CXX_CONFIG_USE_STD_EXPECTED=ON \
      -DIOX2_BB_CXX_CONFIG_USE_STD_OPTIONAL=ON
cmake --build target/ff/cc/build
```

## Using iceoryx2-cxx with custom vocabulary types

In addition to use the `std::optional` and `std::expected` with the iceoryx2 C++
bindings, it is also possible to use custom implementations of these types.

> [!NOTE]
> The custom implementations must be API compatible with the STL counterparts
> and must not depend on iceoryx2-bb-cxx to avoid a circular dependency!

To integrate the custom vocabulary types into iceoryx2, a CMake target for the
`Expected` and `Optional` must be created.

Take a look at the
[custom-vocabulary-types](../../examples/cxx/custom-vocabulary-types) example
as a blueprint to create a CMake package with the required files.

The relevant CMake options are:

* `IOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES`
* `IOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET`

Once the CMake package is installed, iceoryx2 can be configured to use it like
in the following example:

```cmake
cmake -S . \
      -B target/ff/cc/build \
      -DIOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES=ON \
      -DIOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET="my-custom-vocabulary-types" \
      -DCMAKE_PREFIX_PATH=/path/to/my-custom-vocabulary-types/install
cmake --build target/ff/cc/build
```

The CMake target must distribute the following headers:

* `iox2/bb/variation/expected_adaption.hpp`
* `iox2/bb/variation/optional_adaption.hpp`

The header must contain either the code or type aliases for the vocabulary types.
These aliases need to be in the `iox2::bb` namespace.

For the `Optional` these aliases and constants are required:

```cxx
#ifndef MY_OPTIONAL_FOR_ICEORYX2
#define MY_OPTIONAL_FOR_ICEORYX2

#include "my_optional.hpp"

namespace iox2 {
namespace bb {
namespace variation {

template <typename T>
using Optional = my::optional<T>;
using NulloptT = my::nullopt_t;

constexpr NulloptT NULLOPT = my::nullopt;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif // MY_OPTIONAL_FOR_ICEORYX2
```

For the `Expected` these aliases and constants are required:

```cpp
#ifndef MY_EXPECTED_FOR_ICEORYX2
#define MY_EXPECTED_FOR_ICEORYX2

#include "my_expected.hpp"

namespace iox2 {
namespace bb {
namespace variation {

template <typename T, typename E>
using Expected = my::expected<T, E>;
template <typename E>
using Unexpected = my::unexpected<E>;

using InPlaceT = my::in_place_t;
using UnexpectT = my::unexpect_t;

constexpr InPlaceT IN_PLACE = my::in_place;
constexpr UnexpectT UNEXPECT = my::unexpect;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif // MY_EXPECTED_FOR_ICEORYX2
```
