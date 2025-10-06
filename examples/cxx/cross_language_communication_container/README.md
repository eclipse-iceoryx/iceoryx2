# Cross-Language Communication Container

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
>
> **Only fixed-size integers (like `uint8_t`), `float`, `double`, and the**
> **types in the `iceoryx2-bb-container` library are cross-language**
> **compatible!**

This example illustrates how the iceoryx2-bb-container C++ and Rust libraries
can be used for cross-language communication. We use a `StaticVector<uint64_t>`
with a fixed capacity as payload and add a `StaticString` with a fixed capacity
as the user header.

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
./target/ff/cc/build/examples/cxx/cross_language_communication_container/example_cxx_cross_language_communication_container_subscriber
```

### Terminal 2

Run the C++ publisher application:

```sh
./target/ff/cc/build/examples/cxx/cross_language_communication_container/example_cxx_cross_language_communication_container_publisher
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

## How to enable cross-language communication

To enable cross-language communication with containers like `StaticVec`, the
contained type must itself be cross-language compatible. This applies to all
fixed-size integer such as `uint8_t`, `int16_t`, and `float`, and `double`.
Types like `char` and `bool` are not supported because their sizes differ across
languages.

iceoryx2 verifies before connecting that the type name, size, and alignment of
the payload match, preventing the use of non–cross-language-compatible types.
