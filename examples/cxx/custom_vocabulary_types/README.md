# Custom Vocabulary Types

In addition to use the C++ STL types, the `iox2::bb::Expected` and
`iox2::bb::Optional` can be replaced with custom implementations. For detailed
instructions, please have a look at
[doc/user-documentation/how-to-switch-vocabulary-types-for-iceoryx2-cxx](../../doc/user-documentation/how-to-switch-vocabulary-types-for-iceoryx2-cxx).
The custom implementations must be API compatible with the STL counterparts.

In order to use the custom implementation, the vocabulary types must be made
available to the build system. In case of CMake this can be done as following:

```console
cmake -S examples/cxx/custom_vocabulary_types  -B target/ff/cc/custom_vocabulary_types
cmake --build target/ff/cc/custom_vocabulary_types
cmake --install target/ff/cc/custom_vocabulary_types --prefix target/ff/cc/install
```

Now, iceoryx2 needs to be built with the custom vocabulary types:

```console
cargo build --package iceoryx2-ffi-c
cmake -S . \
      -B target/ff/cc/build \
      -DRUST_BUILD_ARTIFACT_PATH=$(pwd)/target/debug \
      -DIOX2_BB_CXX_CONFIG_USE_CUSTOM_VOCABULARY_TYPES=ON \
      -DIOX2_BB_CXX_CONFIG_CUSTOM_VOCABULARY_TYPES_CMAKE_TARGET="custom-vocabulary-types" \
      -DCMAKE_PREFIX_PATH=$(pwd)/target/ff/cc/install
cmake --build target/ff/cc/build
cmake --install target/ff/cc/build --prefix target/ff/cc/install
```

Finally, we use the out-of-tree build of the examples to simulate the integration
into a custom project:

```console
cmake -S examples/cxx \
      -B target/ff/cc/out-of-tree \
      -DCMAKE_PREFIX_PATH=$(pwd)/target/ff/cc/install
cmake --build target/ff/cc/out-of-tree
```
