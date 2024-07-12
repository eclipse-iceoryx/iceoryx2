# Naming Convention

- all constructs start with `iox2_`
- `structs` end with a `_t`
- mutable handles end with a `_mut_h` and are a type definition to a `*mut iox2_foo_storage_t`
- immutable handles end with a `_h` and are a type definition to a `*const iox2_foo_storage_internal_t` which holds the Rust type `Option<Foo>`
- `enums` ends with a `_e`

# Pattern for Type Erasure

The type erasure is usually done in two stages with `iox2_foo_storage_internal_t` and `iox2_foo_storage_t`.

The `iox2_foo_storage_internal_t` is the storage for the Rust type `Option<Foo>` and must match the size and alignment of `Option<Foo>`.
If the internal storage must hold multiple types, the size and alignment is respectively the max value of the types.
The struct is not supposed to be used standalone but always in combination with an `iox2_foo_storage_t`.
Assuming the size is 160 and the alignment is 8, then the storage is defined as following
```rs
#[repr(C)]
#[repr(align(8))] // alignment of Option<Foo>
pub struct iox2_foo_storage_internal_t {
    internal: [u8; 160], // magic number obtained with size_of::<Option<Foo>>()
}
```

The `iox2_foo_storage_t` is the actual storage that is used by the user. It contains the internal storage, a deleter and
optionally further data, e.g. to distinguish between multiple allowed types of `iox2_foo_storage_internal_t`.
```rs
#[repr(C)]
pub struct iox2_foo_storage_t {
    internal: iox2_foo_storage_internal_t,
    deleter: fn(*mut iox2_foo_storage_t),
}
```

A corresponding `iox2_foo_new` or `iox2_foo_builder_create` function initialized the storage. It is recommended to allow
passing a `NULL` pointer to these functions to indicate that the function shall allocate the memory from the heap. A
corresponding `iox2_foo_drop` shall be used to destruct the underlying Rust type and call the deleter function to free the memory.
If the Rust API takes the ownership of `Foo`, the C API will also take the ownership of the handle and `iox2_foo_drop` shall not be
called.

The corresponding handles are defined like this
```rs
pub type iox2_foo_mut_h = *mut iox2_foo_storage_t;
pub type iox2_foo_h = *const iox2_foo_storage_internal_t;
```

The `_mut_h` handle is in general created by a builder and the `_h` handle is in general provided by a function, e.g. as return value.
