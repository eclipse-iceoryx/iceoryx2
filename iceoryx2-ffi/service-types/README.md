# iceoryx2-ffi-service-types

This crate acts as a switch to enable the user to override the service types for
the language bindings. In a future release there will be an option to manually
add more custom service types but but for now, in order to use a customized
service type from a language binding, the definition of these two types need to
be overwritten.

This is done by patching the `iceoryx2-customization-ffi-service-types` crate
with a custom implementation. For more details, see the
[iceoryx2-customization-ffi-service-types README.md](../../customization/iceoryx2-customization-ffi-service-types/README.md).

There is also an [example](../../examples/rust/service_types_customization/README.md)
that shows how service types can be customized.

This is the recommended way to build the bindings:

```sh
cargo build -p iceoryx2-ffi-c --release --config 'patch.crates-io.iceoryx2-customization-ffi-service-types.path="/path/to/crate/with/override"'
cmake -S . -B target/ff/cc/build -DRUST_BUILD_ARTIFACT_PATH=$(pwd)/target/release
cmake --build target/ff/cc/build
```

Alternatively, especially for dev-builds, there is an option to build everything
with cmake:

```sh
cargo build -p iceoryx2-ffi-c --release --config 'patch.crates-io.iceoryx2-customization-ffi-service-types.path="/path/to/crate/with/override"'
cmake -S . -B target/ff/cc/build -DRUST_CUSTOM_FFI_SERVICE_TYPES="/path/to/crate/with/override"
cmake --build target/ff/cc/build
```
