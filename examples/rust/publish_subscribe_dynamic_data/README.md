# Publish-Subscribe With Dynamic Data (Slice Of Shared Memory Compatible Types)

This example illustrates how to send data when the maximum data size cannot
be predetermined and needs to be adjusted dynamically during the service's
runtime. iceoryx2 enables the reallocation of the publisher's data segment,
allowing users to send samples of arbitrary sizes.

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

This example demonstrates a robust publisher-subscriber communication pattern
between two separate processes. A service with the payload type of an `u8` slice
is created, and every publisher can define a slice length hint they support
for communication with `initial_max_slice_len`. The publisher sends a message with
increasing size every second containing a piece of dynamic data. On the receiving
end, the subscriber checks for new data every second.

The subscriber is printing the sample on the console whenever new data arrives.

The `initial_max_slice_len` hint and the `AllocationStrategy` set by the
publisher will define how memory is reallocated when [`Publisher::loan_slice()`]
or [`Publisher::loan_slice_uninit()`] request more memory than it is available.

## How to Run

To observe this dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

```sh
cargo run --example publish_subscribe_dyn_subscriber
```

### Terminal 2

```sh
cargo run --example publish_subscribe_dyn_publisher
```

Feel free to run multiple instances of publisher or subscriber processes
simultaneously to explore how iceoryx2 handles publisher-subscriber
communication efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes run. Take a look at the
> [iceoryx2 config](../../../config) to set the limits globally or at the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
