# Publish-Subscribe With User Header

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

This example illustrates a publisher-subscriber communication pattern between
two separate processes with an additional user header, referred to as a
`CustomHeader`. The publisher sends messages every second, each containing an
incrementing number and the `CustomHeader`, which includes an additional version
number and a timestamp. On the receiving end, the subscriber checks for new data
every second and prints out the received payload and the user header.

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
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe_with_user_header/subscriber.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/publish_subscribe_with_user_header/publisher.py
```

Feel free to run multiple instances of the publisher or subscriber processes
simultaneously to explore how iceoryx2 handles publisher-subscriber
communication efficiently.

> [!TIP]
> You may hit the maximum supported number of ports when too many publisher or
> subscriber processes are running. Check the
> [iceoryx2 config](../../../config) to set the limits globally or refer to the
> [API of the Service builder](https://docs.rs/iceoryx2/latest/iceoryx2/service/index.html)
> to set them for a single service.
