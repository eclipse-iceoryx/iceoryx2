# Cross-language Publish-Subscribe

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

This example illustrates a robust cross-language publisher-subscriber
communication pattern. You can find compatible applications in the
cross-language examples for every language that iceoryx2 supports. The publisher
applications of the cross-language examples send a message every second, each
containing `TransmissionData` and the `CustomHeader`. On the receiving end, the
subscriber applications of the cross-language examples print the received
payload and the user header to the console whenever new data arrives.

## How to Run

To observe the dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

Run the Rust subscriber application:

```sh
cargo run --example publish_subscribe_cross_language_subscriber
```

### Terminal 2

Run the Rust publisher application:

```sh
cargo run --example publish_subscribe_cross_language_publisher
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

To allow cross-language communication involving Rust applications, iceoryx2
provides the possibility to customize these type names by implementing
`ZeroCopySend::type_name()` for the payload and user header types or by setting
the helper attribute `type_name` when `ZeroCopySend` is derived for the types,
e.g.

``` rust
#[derive(ZeroCopySend)]
#[type_name(TransmissionData)]
#[repr(C)]
pub struct TransmissionData {
  // ...
}
```

When the type names are set to the same value, and the structure has the same
memory layout, the Rust applications and applications written in other supported
languages can communicate.

> [!TIP]
> You can also send dynamic data between Python, Rust and C++ applications (see
> [Publish-Subscribe With Dynamic Data](../publish_subscribe_dynamic_data)). If
> you send `iox::Slice`s of `(u)int{8|16|32|64}_t`, `float`, `double` or
> `bool`, the payload type name is automatically translated to the Rust
> equivalent. For other slice types, you have to set `IOX2_TYPE_NAME` for the
> inner type to the Rust equivalent to enable the communication.
