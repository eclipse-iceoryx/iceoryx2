# Blackboard

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
>
> **Only fixed-size integers (like `u8`), floating-point types (`f32` and**
> **`f64`), and the types in the `iceoryx2-bb-container` library are**
> **cross-language compatible!**

This example illustrates the blackboard messaging pattern, a key-value
repository in shared memory. Each communication participant can access exactly
the entries it needs instead of the whole repository, making it useful for
sharing a global configuration or state, for example.

> [!IMPORTANT]
> In addition to the shared memory related requirements mentioned above, the
> keys and values stored in the blackboard must implement `Copy`. To be able to
> store and retrieve keys in the blackboard, the key must also implement `Eq`.

In this example, one writer updates the values in the blackboard every second
and a reader reads and prints them to the console. The key-value pairs must be
defined via the the service builder:

```rust
node.service_builder(&service_name)
    .blackboard_creator::<BlackboardKey>()
    .add::<i32>(key_0, 3)
    .add::<f64>(key_1, INITIAL_VALUE_1)
    .create()?;
```

## How to Run

To observe the blackboard messaging pattern in action, open two separate
terminals and execute the following commands:

### Terminal 1

```sh
cargo run --example blackboard_creator
```

### Terminal 2

```sh
cargo run --example blackboard_opener
```

Feel free to run multiple instances of reader processes simultaneously but note
that the `blackboard_creator` must run first to create the blackboard service
with the key-value pairs and that there can be only one writer.
