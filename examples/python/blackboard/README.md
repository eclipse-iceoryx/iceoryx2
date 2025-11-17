# Blackboard

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
> cause undefined behavior and may result in segmentation faults.
>
> **Only fixed-size integers (like `ctypes.c_uint8_t`), `ctypes.c_float`,**
> **`ctypes.c_double`, and the types in the `iceoryx2-bb-container` library**
> **are cross-language compatible!**

This example illustrates the blackboard messaging pattern, a key-value
repository in shared memory. Each communication participant can access exactly
the entries it needs instead of the whole repository, making it useful for
sharing a global configuration or state, for example.

> [!IMPORTANT]
> In addition to the shared memory related requirements mentioned above, the
> keys and values stored in the blackboard must be trivially copyable. To be
> able to store and retrieve keys in the blackboard, the key must implement
> __eq__.

In this example, one writer updates the values in the blackboard every second
and a reader reads and prints them to the console.

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

To observe the blackboard messaging pattern in action, open two separate
terminals and execute the following commands:

### Terminal 1

```sh
poetry --project iceoryx2-ffi/python run python examples/python/blackboard/creator.py
```

### Terminal 2

```sh
poetry --project iceoryx2-ffi/python run python examples/python/blackboard/opener.py
```

Feel free to run multiple instances of reader processes simultaneously but note
that the `blackboard_creator` must run first to create the blackboard service
with the key-value pairs and that there can be only one writer.
