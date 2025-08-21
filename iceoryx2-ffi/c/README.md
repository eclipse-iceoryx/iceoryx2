<!-- markdownlint-disable MD044 'c' needs to be lower-case -->
# iceoryx2-ffi-c
<!-- markdownlint-enable MD044 -->

## Naming Convention

* all constructs start with `iox2_`
* `structs` end with a `_t`
* owning handles end with a `_h` and are a type definition to a
  `struct iox2_foo_h_t;` as `pub type iox2_foo_h = *mut iox2_foo_h_t`
* non-owning handles end with a `_h_ref` and are a type definition to a
  `iox2_foo_h` as
  `pub type iox2_foo_h_ref = *const iox2_foo_h`
* immutable pointer to the Rust type end with a `_ptr` and are a type definition
  like `pub type iox2_foo_ptr = *const Foo`
* mutable pointer to the Rust type end with a `_ptr_mut` and are a type
  definition like `pub type iox2_foo_ptr_mut = *mut Foo`
* `enums` ends with a `_e`

## Pattern for Type Erasure

The type erasure is usually done in two stages with `iox2_foo_storage_t` and
`iox2_foo_t`.

The `iox2_foo_storage_t` is the storage for the Rust type `Option<Foo>` and must
match the size and alignment of `Option<Foo>`. If the internal storage must hold
multiple types, a union can be used. The struct is not supposed to be used
standalone but always in combination with an `iox2_foo_t`. Assuming the size is
160 and the alignment is 8, then the storage is defined as following

```rs
#[repr(C)]
#[repr(align(8))] // alignment of Option<Foo>
pub struct iox2_foo_storage_t {
    internal: [u8; 160], // magic number obtained with size_of::<Option<Foo>>()
}
```

The `iox2_foo_t` is the actual type that is used by the user. It contains the
internal storage, a deleter and optionally further data, e.g. to distinguish
between multiple allowed types of `iox2_foo_storage_t`.

```rs
#[repr(C)]
pub struct iox2_foo_t {
    /// cbindgen:rename=internal
    foo: iox2_foo_storage_t,
    deleter: fn(*mut iox2_foo_t),
}
```

A corresponding `iox2_foo_new` or `iox2_foo_builder_create` function initialized
the storage. It is recommended to allow passing a `NULL` pointer to these
functions to indicate that the function shall allocate the memory from the heap.
A corresponding `iox2_foo_drop` shall be used to destruct the underlying Rust
type and call the deleter function to free the memory. If the Rust API takes the
ownership of `Foo`, the C API will also take the ownership of the handle and
`iox2_foo_drop` shall not be called.

When the owning handle is passed to a function, the ownership of the underlying
data is moved to that specific function and the `*_h` handles as well as all the
`*_ptr` related to that handle are invalid. Accessing the handles or pointer
afterwards lead to undefined behavior. The only exception are the `iox2_cast_*`
functions which can be used to get `_ptr` and `_ptr_mut` pointer the the Rust
type or a non-owning `_h_ref` handle to the C struct.

The corresponding handle and pointer are defined like this

```rs
pub struct iox2_foo_h_t;
pub type iox2_foo_h = *mut iox2_foo_h_t;
pub type iox2_foo_h_ref = *const iox2_foo_h;

pub type iox2_foo_ptr = *const Foo;

pub type iox2_foo_ptr_mut = *mut Foo;
```

The `_h` handle is in general created by a builder and the `_ptr` pointer are in
general provided by a function, e.g. as return value.

The `src/node_name.rs` file can be used as a more comprehensive example on how
to implement an FFI binding for a specific type.

## Opaque Types

In order to prevent symbol pollution in C, all types need to be prefixed with
`iox2_` or renamed via `cbindgen.toml` if they originate outside of this crate.
The opaque types additionally need to be manually forward-declared in
`cbindgen.toml` since cbindgen does not do it automatically.

* renaming is done in `[export.rename]` with `"Foo" = "iox2_foo_ptr_t"`
* forward declaration is done in the `after_includes` section with
  `typedef struct iox2_foo_ptr_t iox2_foo_ptr_t;`

<!-- markdownlint-disable MD044 'c' needs to be lower-case -->
## Passing feature flags to the iceoryx2-ffi-c crate
<!-- markdownlint-enable MD044 -->

To pass `iceoryx2` feature flags to the `iceoryx2-ffi-c` crate, one needs to
prefix the feature with `iceoryx2/`, e.g. `--features iceoryx2/libc_platform.`.

## Why the folder structure with 'api' and 'test'

As it turned out `cdylib`s do not play well with integration tests. The `cdylib`
is build with `panic="abort"` but the tests require `panic="unwind"`. This
results in building the lib twice if there are integration tests and leads to
the following warning and eventually to build failures.

<!-- markdownlint-disable -->

> warning: output filename collision.
> The lib target `iceoryx2_ffi-c` in package `iceoryx2-ffi-c vX.Y.Z (C:\Users\ekxide\iceoryx2\iceoryx2-ffi\c)`
> has the same output filename as the lib target `iceoryx2_ffi-c` in package
> `iceoryx2-ffi-c vX.Y.Z (C:\Users\ekxide\iceoryx2\iceoryx2-ffi\c)`.
> Colliding filename is: C:\Users\ekxide\iceoryx2\target\release\deps\iceoryx2_ffi_c.lib
> The targets should have unique names.
> Consider changing their names to be unique or compiling them separately.
> This may become a hard error in the future; see <https://github.com/rust-lang/cargo/issues/6313>.

<!-- markdownlint-enable -->

As a workaround, the integrationtests are placed in the module. This would give
access to private API though. To circumvent this problem, only `pub(super)`
shall be used if an API needs to be available in other modules but not
`pub(crate)`. With the chosen folder structure the tests can again only be
written as whitebox tests.
