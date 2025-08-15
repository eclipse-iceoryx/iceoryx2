# Publish-Subscribe With Dynamic Data (Slice Of Shared Memory Compatible Types)

This example demonstrates how to send data when the maximum data size cannot
be predetermined and needs to be adjusted dynamically during the service's
runtime. iceoryx2 enables the reallocation of the publisher's data segment,
allowing users to send samples of arbitrary sizes.

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
>     * therefore, only `ctypes` and `ctypes.Structure` can be transferred
> * not use pointers to manage their internal structure
>
> Any other python data type, except `ctypes` or `ctypes.Structure`s, like will
> cause undefined behavior and may result in segmentation faults. Take a look
> at the [publish subscribe example](../publish_subscribe) to see how `ctypes`
> can be transferred.

This example demonstrates a robust publisher-subscriber communication pattern
between two separate processes. A service with the payload type of an
`ctypes.c_uint8` `iox2.Slice`
is created, and every publisher can define a slice length hint they support
for communication with `initial_max_slice_len`. The publisher sends a message with
increasing size every second containing a piece of dynamic data. On the receiving
end, the subscriber checks for new data every second.

The subscriber is printing the sample on the console whenever new data arrives.

The `initial_max_slice_len` hint and the `AllocationStrategy` set by the
publisher will define how memory is reallocated when [`Publisher::loan_slice()`]
or [`Publisher::loan_slice_uninit()`] request more memory than it is available.

## How to Build

Before proceeding, a virtual environment with all dependencies needs to be
created. You can find the detailed instructions in the
[Python Examples Readme](../README.md).

```sh
poetry --project iceoryx2-ffi/python install
```

Then, the iceoryx2 python bindings can be built and installed into the virtual
environment:

```sh
poetry --project iceoryx2-ffi/python run maturin develop --manifest-path iceoryx2-ffi/python/Cargo.toml --target-dir target/ff/python
```

## How to Run

To observe this dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

```sh
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe_dynamic_data/subscriber.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe_dynamic_data/publisher.py
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
