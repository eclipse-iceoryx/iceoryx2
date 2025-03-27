# Complex Data Types

## Running The Example

> [!CAUTION]
> Every payload you transmit with iceoryx2 must be compatible with shared
> memory. Specifically, it must:
>
> * be self contained, no heap, no pointers to external sources
> * have a uniform memory representation -> `#[repr(C)]`
> * not use pointers to manage their internal structure
> * the type and its members don't implement `Drop` explicitly

This example demonstrates how the zero-copy compatible versions of `Vec` or
`String` can be sent.
The crate `iceoryx2-bb-container` provides versions that are shared memory
compatible like `FixedSizeVec` and `FixedSizeByteString`.

**Note**:** There also exist more advanced types called `Relocatable**`, that
will become the basic building blocks for dynamic-sized messages in iceoryx2, so
that the user is not forced to provide a capacity at compile-time.

To see the example in action, open a terminal and enter:

```sh
cargo run --example complex_data_types
```

**Note:** The example can be started up to 16 times in parallel. The subscriber
would then receive the samples from every publisher from every running instance.

## How To Define Custom Data Types

1. Ensure to only use data types suitable for shared memory communication like
   pod-types (plain old data, e.g. `usize`, `f32`, ...) or explicitly
   shared-memory compatible containers like some of the constructs in the
   `iceoryx2-bb-containers`.
2. Add `#[repr(C`)]` to your custom data type so that it has a uniform memory
   representation.

   ```rust
    #[repr(C)]
    struct MyDataType {
        //....
    }
   ```

3. **Do not use pointers, or data types that are not self-contained or use
   pointers for their internal management!**
