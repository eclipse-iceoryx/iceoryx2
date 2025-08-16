# Cross-language Publish-Subscribe

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure

This example illustrates a robust cross-language publisher-subscriber
communication pattern. You can find compatible applications in the
cross-language examples for every language that iceoryx2 supports. The publisher
applications of the cross-language examples send a message every second, each
containing `TransmissionData` and the `CustomHeader`. On the receiving end, the
subscriber applications of the cross-language examples print the received
payload and the user header to the console whenever new data arrives.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C Examples Readme](../README.md).

When you want to run the C publisher and subscriber applications, you first have
to build the C examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```

## How to Run

To observe the dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the C subscriber application:

```sh
./target/ff/cc/build/examples/c/publish_subscribe_cross_language/example_c_cross_language_subscriber
```

### Terminal 2

Run the C publisher application:

```sh
./target/ff/cc/build/examples/c/publish_subscribe_cross_language/example_c_cross_language_publisher
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

For C applications, these type names must be set with
`iox2_service_builder_pub_sub_set_payload_type_details` and
`iox2_service_builder_pub_sub_set_user_header_type_details` before creating the
service.

When the type names are set to the same value, and the structure has the same
memory layout, the C applications and applications written in other supported
languages can communicate.
