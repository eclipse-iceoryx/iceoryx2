# Publish-Subscribe Between C++ and Rust applications

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
between a C++ publisher and a Rust subcriber application. The C++ publisher
application sends a message every second, each containing `TransmissionData` and
the `CustomHeader`. On the receiving end, the Rust subscriber application checks
for new data every second and prints out the received payload and the user
header.

First you have to build the C++ examples:

```sh
cmake -S . -B target/ffi/build -DBUILD_EXAMPLES=ON
cmake --build target/ffi/build
```

To observe this dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the Rust subscriber application:

```sh
cargo run --example publish_subscribe_cxx2rust_subscriber
```

### Terminal 2

Run the C++ publisher application from the
[publish_subscribe_cxx2rust example](../../cxx/publish_subscribe_cxx2rust):

```sh
./target/ffi/build/examples/cxx/publish_subscribe_cxx2rust/example_cxx_publish_subscribe_cxx2rust_publisher
```

Feel free to also run the C++ subscriber and the Rust publisher applications
simultaneously to explore how iceoryx2 handles cross-language
publisher-subscriber communication efficiently.

### Terminal 3

Run the C++ subscriber application from the
[publish_subcribe_cxx2rust example](../../cxx/publish_subscribe_cxx2rust):

```sh
./target/ffi/build/examples/cxx/publish_subscribe_cxx2rust/example_cxx_publish_subscribe_cxx2rust_subscriber
```

### Terminal 4

Run the Rust publisher application:

```sh
cargo run --example publish_subscribe_cxx2rust_publisher
```

## How to enable publish-subscribe communication between Rust and C++ applications

To communicate with each other, publisher and subscriber applications must share
the same service configuration, including the payload and the user header type
name. Usually, the internally derived type names depend on the used programming
language. To allow communication between C++ and Rust, iceoryx2 provides the
possibility to customize the payload and the user header type name by setting
`TYPE_NAME` in the sent data struct and user header, e.g.

```cxx
struct TransmissionData {
    static constexpr const char* TYPE_NAME = "examples_common::transmission_data::TransmissionData";
    std::int32_t x;
    std::int32_t y;
    double funky;
};

struct CustomHeader {
    static constexpr const char* TYPE_NAME = "examples_common::custom_header::CustomHeader";
    int32_t version;
    uint64_t timestamp;
};
```

_Note:_ `TYPE_NAME` is currently only taken into account on the C++ side.

When `TYPE_NAME` is set to the same type name used in the Rust application, and
the structure has the same memory layout, the C++ and the Rust applications can
communicate.

_Hint:_ You can determine the type names on the Rust side with
`core::any::type_name()`.

For {u}int{8|16|32|64}_t, float, double and bool, you don't need to provide
`TYPE_NAME` for the payload as these types are automatically translated into the
Rust pendants.

You can also send dynamic data between C++ and Rust applications (see
[Publish-Subscribe With Dynamic Data](../publish_subscribe_dynamic_data)). If
you send `iox::Slice`s of {u}int{8|16|32|64}_t, float, double or bool, the
payload type name is automatically translated to the Rust pendant. For other
slice types, you have to set `TYPE_NAME` for the inner type to the Rust pendant
to enable the communication.
