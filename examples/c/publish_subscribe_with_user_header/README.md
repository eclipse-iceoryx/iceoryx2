# Publish-Subscribe With User Header

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure

This example illustrates a publisher-subscriber communication pattern between
two separate processes with an additional user header, referred to as a
`CustomHeader`. The publisher sends messages every second, each containing an
incrementing number and the `CustomHeader`, which includes an additional version
number and a timestamp. On the receiving end, the subscriber checks for new data
every second and prints out the received payload and the user header.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C Examples Readme](../README.md).

First you have to build the C examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```

## How to Run

To observe this dynamic communication in action, open two separate terminals and
execute the following commands:

### Terminal 1

```sh
./target/ff/cc/build/examples/c/publish_subscribe_with_user_header/example_c_with_user_header_subscriber
```

### Terminal 2

```sh
./target/ff/cc/build/examples/c/publish_subscribe_with_user_header/example_c_with_user_header_publisher
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
