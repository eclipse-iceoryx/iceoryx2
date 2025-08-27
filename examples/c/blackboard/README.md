# Blackboard

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation, ensuring that shared structs have the
>     same data layout
> * not use pointers to manage their internal structure

This example illustrates the blackboard messaging pattern. A writer updates the
values in the blackboard every second and a reader reads and prints them to the
console.

## How to Build

Before proceeding, all dependencies need to be installed. You can find
instructions in the [C Examples Readme](../README.md).

First you have to build the C examples:

```sh
cmake -S . -B target/ff/cc/build -DBUILD_EXAMPLES=ON -DBUILD_CXX=OFF
cmake --build target/ff/cc/build
```

## How to Run

To observe the blackboard messaging pattern in action, open two separate
terminals and execute the following commands:

### Terminal 1

```sh
./target/ff/cc/build/examples/c/blackboard/example_c_creator
```

### Terminal 2

```sh
./target/ff/cc/build/examples/c/blackboard/example_c_opener
```

Feel free to run multiple instances of reader processes simultaneously but note
that the `blackboard_creator` must run first to create the blackboard service
with the key-value pairs and that there can be only one writer.
