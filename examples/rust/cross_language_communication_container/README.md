# Cross-Language Communication Container

> [!CAUTION]
> Every payload you transmit with iceoryx2 must implement [`ZeroCopySend`] to
> be compatible with shared memory.
> Usually, you can use the derive-macro `#[derive(ZeroCopySend)]` for most
> types. If you implement it manually you must ensure that the payload type:
>
> * is self contained, no heap, no pointers to external sources
> * has a uniform memory representation -> `#[repr(C)]`
> * does not use pointers to manage their internal structure
> * and its members don't implement `Drop` explicitly
> * has a `'static` lifetime
>
> Data types like `String` or `Vec` will cause undefined behavior and may
> result in segmentation faults. We provide alternative data types that are
> compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.
>
> **Only fixed-size integers (like `u8`), floating-point types (`f32` and**
> **`f64`), and the types in the `iceoryx2-bb-container` library are**
> **cross-language compatible!**

This example illustrates how the iceoryx2-bb-container C++ and Rust libraries
can be used for cross-language communication. We use a `StaticVec<u64>` with a
fixed capacity as payload and add a `StaticString` with a fixed capacity as the
user header.

## How to Run

To observe the dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the Rust subscriber application:

```sh
cargo run --example cross_language_communication_container_subscriber
```

### Terminal 2

Run the Rust publisher application:

```sh
cargo run --example cross_language_communication_container_publisher
```

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes are running. Check the [iceoryx2 config](../../../config)
> to set the limits globally or refer to the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.

## How to enable cross-language communication

To enable cross-language communication with containers like `StaticVec`, the
contained type must itself be cross-language compatible. This applies to all
fixed-size integer and floating-point types such as `u8`, `i16`, `f32`, and
`f64`. Types like `char` and `bool` are not supported because their sizes differ
across languages.

iceoryx2 verifies before connecting that the type name, size, and alignment of
the payload match, preventing the use of non–cross-language-compatible types.
