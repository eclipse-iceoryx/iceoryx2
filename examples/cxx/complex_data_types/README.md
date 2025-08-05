# Complex Data Types

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * not use pointers to manage their internal structure
> * must be trivially destructible, see `std::is_trivially_destructible`

This example demonstrates how the zero-copy compatible versions of
`std::vector` or `std::string` can be sent.
The library
[iceoryx_hoofs](https://github.com/eclipse-iceoryx/iceoryx/tree/main/iceoryx_hoofs)
provides versions that are shared memory compatible like the
`iox::string` and the `iox::vector`.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

To see the example in action, open a terminal and enter:

```sh
./target/ff/cc/build/examples/cxx/complex_data_types/example_cxx_complex_data_types
```

**Note:** The example can be started up to 16 times in parallel. The subscriber
would then receive the samples from every publisher from every running instance.

## How To Define Custom Data Types

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `usize`, `f32`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx-hoofs`.
2. **Do not use pointers, or data types that are not self-contained or use
   pointers for their internal management!**
