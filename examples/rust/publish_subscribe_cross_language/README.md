# Cross-language Publish-Subscribe

## Running The Example

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation -> `#[repr(C)]`
> * not use pointers to manage their internal structure
>
> Data types like `String` or `Vec` will cause undefined behavior and may
> result in segmentation faults. We provide alternative data types that are
> compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.

This example illustrates a robust publisher-subscriber communication pattern
between a Rust publisher and a C++ subcriber application. The Rust publisher
application sends a message every second, each containing `TransmissionData` and
the `CustomHeader`. On the receiving end, the C++ subscriber application prints
the received payload and the user header to the console whenever new data
arrives.

First you have to build the C++ examples:

```sh
cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=ON
cmake --build target/ffi/build
```

To observe this dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the C++ subscriber application from the
[C++ cross-language publish-subcribe example](../../cxx/publish_subscribe_cross_language):

```sh
./target/ffi/build/examples/cxx/publish_subscribe_cross_language/example_cxx_publish_subscribe_cross_language_subscriber
```

### Terminal 2

Run the Rust publisher application:

```sh
cargo run --example publish_subscribe_cross_language_publisher
```

Feel free to also run the Rust subscriber and the C++ publisher applications
simultaneously to explore how iceoryx2 handles cross-language
publisher-subscriber communication efficiently.

### Terminal 3

Run the Rust subscriber application:

```sh
cargo run --example publish_subscribe_cross_language_subscriber
```

### Terminal 4

Run the C++ publisher application from the
[C++ cross-language publish-subscribe example](../../cxx/publish_subscribe_cross_language):

```sh
./target/ffi/build/examples/cxx/publish_subscribe_cross_language/example_cxx_publish_subscribe_cross_language_publisher
```

You can also communicate with the C publisher and subscriber applications from
the
[C cross-language publish-subscribe example](../../c/publish_subscribe_cross_language).

You may hit the maximum supported number of ports when too many publisher or
subscriber processes are running. Check the [iceoryx2 config](../../../config)
to set the limits globally or refer to the
[API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
to set them for a single service.

## How to enable publish-subscribe communication between Rust, C++ and C applications

To communicate with each other, publisher and subscriber applications must share
the same service configuration, including the payload and the user header type
name.

For C++ and Rust applications, the internally derived type names
usually depend on the used programming language. To allow cross-language
communication involving C++ applications, iceoryx2 provides the possibility to
customize the payload and the user header type name by setting `IOX2_TYPE_NAME` in
the sent C++ data struct and user header, e.g.

```cxx
struct TransmissionData {
    static constexpr const char* IOX2_TYPE_NAME = "examples_common::transmission_data::TransmissionData";
    std::int32_t x;
    std::int32_t y;
    double funky;
};

struct CustomHeader {
    static constexpr const char* IOX2_TYPE_NAME = "examples_common::custom_header::CustomHeader";
    int32_t version;
    uint64_t timestamp;
};
```

For C applications, these type names must be set with
`iox2_service_builder_pub_sub_set_payload_type_details` and
`iox2_service_builder_pub_sub_set_user_header_type_details` before creating the
service.

_Note:_ The type name can't currently be set for Rust applications. You can
determine the type names on the Rust side with `core::any::type_name()` and use
these as the type names in C and C++ applications.

When the type names are set to the same value, and the structure has the same
memory layout, the Rust, C++ and the C applications can communicate.

For the C++ types (u)int{8|16|32|64}_t, float, double and bool, you don't need
to provide `IOX2_TYPE_NAME` for the payload as these types are automatically
translated into the Rust equivalents.

You can also send dynamic data between Rust and C++ applications (see
[Publish-Subscribe With Dynamic Data](../publish_subscribe_dynamic_data)). If
you send `iox::Slice`s of (u)int{8|16|32|64}_t, float, double or bool, the
payload type name is automatically translated to the Rust equivalent. For other
slice types, you have to set `IOX2_TYPE_NAME` for the inner type to the Rust
equivalent to enable the communication.
