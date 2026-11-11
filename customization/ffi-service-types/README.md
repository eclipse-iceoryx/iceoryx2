# iceoryx2 customization for FFI service types

In order to override the built-in service types for the FFI bindings, the
cargo patch mechanism can be used to inject a custom crate with custom
service types into the build chain of the FFI bindings.

This crate will be used in combination with the `iceoryx2-ffi-service-types`
crate which will check if the `USE_CUSTOM` flag is true and use the service
types defined in the respective crate if it is found true.

The custom crate must have the same name and version as the
`iceoryx2-customization-ffi-service-types` crate, in order to override the
dummy package from this crate.

In order to still be able to use a version number for the custom crate, it is
recommended to have a small first crate without logic for the override and
re-export the logic from a second crate with the actual implementation. The
version number of the second crate in the dependencies gives a hint about
the actual version.

To use the customization override, the cargo `--config patch....` option can be
used to insert the custom implementation into the build chain.

```sh
cargo build -p iceoryx2-ffi-c --config 'patch.crates-io.iceoryx2-customization-ffi-service-types.path="/path/to/crate/with/override"'
```

## Example

The `Cargo.toml` need to look like this:

```toml[package]
name = "iceoryx2-customization-ffi-service-types"
version = "0.0.1"

[dependencies]
my-ffi-service-types = 0.8.15
```

The `lib.rs` needs to export the service types and set the `USE_CUSTOM` flag:

```rust
pub const USE_CUSTOM: bool = true;

pub type IpcService = my_ffi_service_types::ipc::Service;
pub type LocalService = my_ffi_service_types::local::Service;
```

The `my-ffi-service-types` is the second crate with the actual logic and can
have a custom version number. These two types need to be re-exported:
