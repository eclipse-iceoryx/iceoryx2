# Naming Convention

- all constructs start with `iox2_`
- `structs` end with a `_t`
- owning handles end with a `_h` and are a type definition to a `struct iox2_foo_h_t;` as `pub type iox2_foo_h = *mut iox2_foo_h_t`
- non-owning handles end with a `_ref_h` and are a type definition to a `struct iox2_foo_ref_h_t;` as `pub type iox2_foo_ref_h = *mut iox2_foo_ref_h_t`
- immutable pointer to the Rust type end with a `_ptr` and are a type definition to a `struct iox2_foo_ptr_t;` as `pub type iox2_foo_ptr = *const iox2_foo_ptr_t`
- mutable pointer to the Rust type end with a `_mut_ptr` and are a type definition to a `struct iox2_foo_mut_ptr_t;` as `pub type iox2_foo_mut_ptr = *mut iox2_foo_mut_ptr_t`
- `enums` ends with a `_e`

# Pattern for Type Erasure

The type erasure is usually done in two stages with `iox2_foo_storage_t` and `iox2_foo_t`.

The `iox2_foo_storage_t` is the storage for the Rust type `Option<Foo>` and must match the size and alignment of `Option<Foo>`.
If the internal storage must hold multiple types, the size and alignment is respectively the max value of the types.
The struct is not supposed to be used standalone but always in combination with an `iox2_foo_t`.
Assuming the size is 160 and the alignment is 8, then the storage is defined as following
```rs
#[repr(C)]
#[repr(align(8))] // alignment of Option<Foo>
pub struct iox2_foo_storage_t {
    internal: [u8; 160], // magic number obtained with size_of::<Option<Foo>>()
}
```

The `iox2_foo_t` is the actual type that is used by the user. It contains the internal storage, a deleter and
optionally further data, e.g. to distinguish between multiple allowed types of `iox2_foo_storage_t`.
```rs
#[repr(C)]
pub struct iox2_foo_t {
    /// cbindgen:rename=internal
    foo: iox2_foo_storage_t,
    deleter: fn(*mut iox2_foo_t),
}
```

A corresponding `iox2_foo_new` or `iox2_foo_builder_create` function initialized the storage. It is recommended to allow
passing a `NULL` pointer to these functions to indicate that the function shall allocate the memory from the heap. A
corresponding `iox2_foo_drop` shall be used to destruct the underlying Rust type and call the deleter function to free the memory.
If the Rust API takes the ownership of `Foo`, the C API will also take the ownership of the handle and `iox2_foo_drop` shall not be
called.

When the owning handle is passed to a function, the ownership of the underlying data is moved to that specific function and the `*_h` handles
as well as all the `*_ptr` related to that handle are invalid. Accessing the handles or pointer afterwards lead to undefined behavior.
The only exception are the `iox2_cast_*` functions which can be used to get `_ptr` and `_mut_ptr` pointer the the Rust type or a non-owning `_ref_h` handle to the C struct.

The corresponding handle and pointer are defined like this
```rs
pub struct iox2_foo_h_t;
pub type iox2_foo_h = *mut iox2_foo_h_t;

pub struct iox2_foo_ref_h_t;
pub type iox2_foo_ref_h = *mut iox2_foo_ref_h_t;

pub struct iox2_foo_ptr_t;
pub type iox2_foo_ptr = *const iox2_foo_ptr_t;

pub struct iox2_foo_mut_ptr_t;
pub type iox2_foo_mut_ptr = *mut iox2_foo_mut_ptr_t;
```

The `_h` handle is in general created by a builder and the `_ptr` pointer ar in general provided by a function, e.g. as return value.

The `src/node_name.rs` file can be used as a more comprehensive example on how to implement an FFI binding for a specific type.
