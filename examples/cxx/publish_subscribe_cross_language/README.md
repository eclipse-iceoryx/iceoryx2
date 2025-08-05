# Cross-language Publish-Subscribe

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure
> * must be trivially destructible, see `std::is_trivially_destructible`
>
> Data types like `std::string` or `std::vector` will cause undefined behavior
> and may result in segmentation faults. We provide alternative data types
> that are compatible with shared memory. See the
> [complex data type example](../complex_data_types) for guidance on how to
> use them.

This example illustrates a robust cross-language publisher-subscriber
communication pattern. You can find compatible applications in the
cross-language examples for every language that iceoryx2 supports. The publisher
applications of the cross-language examples send a message every second, each
containing `TransmissionData` and the `CustomHeader`. On the receiving end, the
subscriber applications of the cross-language examples print the received
payload and the user header to the console whenever new data arrives.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

When you want to run the C++ publisher and subscriber applications, you first
have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

To observe the dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the C++ subscriber application:

```sh
./target/ff/cc/build/examples/cxx/publish_subscribe_cross_language/example_cxx_publish_subscribe_cross_language_subscriber
```

### Terminal 2

Run the C++ publisher application:

```sh
./target/ff/cc/build/examples/cxx/publish_subscribe_cross_language/example_cxx_publish_subscribe_cross_language_publisher
```

Feel free to also run the subscriber and publisher applications from other
cross-language examples simultaneously to explore how iceoryx2 handles
publisher-subscriber communication between applications written in different
languages efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes are running. Check the [iceoryx2 config](../../../config)
> to set the limits globally or refer to the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.

## How to enable cross-language publish-subscribe communication

To communicate with each other, publisher and subscriber applications must share
the same service configuration, including the payload and the user header type
name.

To allow cross-language communication involving C++ applications, iceoryx2
provides the possibility to customize the payload and the user header type name
by setting `IOX2_TYPE_NAME` in the sent C++ data struct and user header, e.g.

```cxx
struct TransmissionData {
    static constexpr const char* IOX2_TYPE_NAME = "TransmissionData";
    std::int32_t x;
    std::int32_t y;
    double funky;
};

struct CustomHeader {
    static constexpr const char* IOX2_TYPE_NAME = "CustomHeader";
    int32_t version;
    uint64_t timestamp;
};
```

When the type names are set to the same value, and the structure has the same
memory layout, the C++ applications and applications written in other supported
languages can communicate.

> [!NOTE]
> For the communication with Rust applications, you don't need to provide
> `IOX2_TYPE_NAME` for `(u)int{8|16|32|64}_t`, `float`, `double` and `bool`
> payloads.
> These types are automatically translated into the Rust equivalents.

You can also send dynamic data between Python, C++ and Rust applications (see
[Publish-Subscribe With Dynamic Data](../publish_subscribe_dynamic_data)). If
you send `iox::Slice`s of `(u)int{8|16|32|64}_t`, `float`, `double` or `bool`,
the payload type name is automatically translated to the Rust equivalent. For
other slice types, you have to set `IOX2_TYPE_NAME` for the inner type to the
Rust equivalent to enable the communication.
