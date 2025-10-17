# Blackboard

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

This example illustrates the blackboard messaging pattern. A writer updates the
values in the blackboard every second and a reader reads and prints them to the
console.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C++ Examples Readme](../README.md).

First you have to build the C++ examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON
cmake --build target/ff/cc/build
```

## How to Run

To observe the blackboard messaging pattern in action, open two separate
terminals and execute the following commands:

### Terminal 1

```sh
./target/ff/cc/build/examples/cxx/blackboard/example_cxx_blackboard_creator
```

### Terminal 2

```sh
./target/ff/cc/build/examples/cxx/blackboard/example_cxx_blackboard_opener
```

Feel free to run multiple instances of reader processes simultaneously but note
that the `blackboard_creator` must run first to create the blackboard service
with the key-value pairs and that there can be only one writer.
