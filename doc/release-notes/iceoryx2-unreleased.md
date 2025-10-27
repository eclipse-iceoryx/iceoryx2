# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* `iox2 config explain` cli command for config descriptions
  [#832](https://github.com/eclipse-iceoryx/iceoryx2/issues/832)
* Add traits to facilitate implementation of custom tunnelling mechanisms
  [#845](https://github.com/eclipse-iceoryx/iceoryx2/issues/845)
* Add a C++ string container type with fixed compile-time capacity
  [#938](https://github.com/eclipse-iceoryx/iceoryx2/issues/938)
* Add a C++ vector container type with fixed compile-time capacity
  [#951](https://github.com/eclipse-iceoryx/iceoryx2/issues/951)
* Use `epoll` instead of `select` for the `WaitSet` on Linux
  [#961](https://github.com/eclipse-iceoryx/iceoryx2/issues/961)
* Add a Rust vector type with fixed compile-time capacity which has the same
  memory layout as the C++ vector
  [#1073](https://github.com/eclipse-iceoryx/iceoryx2/issues/1073)
* Add a Rust string type with fixed compile-time capacity which has the same
  memory layout as the C++ vector
  [#1075](https://github.com/eclipse-iceoryx/iceoryx2/issues/1075)
* Add unchecked, compile time const creation functions to `SemanticString` and
  system types like, `FileName`, `Path`, `FilePath`, ...
  [#1109](https://github.com/eclipse-iceoryx/iceoryx2/issues/1109)
* Add conformance test suite to be able to test out-of-tree extensions
  [#1021](https://github.com/eclipse-iceoryx/iceoryx2/issues/1021)
* Implement `Copy` for `StaticString`, `SemanticString` and system types
  [#1114](https://github.com/eclipse-iceoryx/iceoryx2/issues/1114)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove duplicate entries in `iox2` command search path to prevent discovered
  commands from being listed multiple times
    [#1045](https://github.com/eclipse-iceoryx/iceoryx2/issues/1045)
* LocalService in C language binding uses IPC configuration
    [#1059](https://github.com/eclipse-iceoryx/iceoryx2/issues/1059)
* Trait `std::fmt::Debug` is not implemented for `sigset_t` in libc
    [#1087](https://github.com/eclipse-iceoryx/iceoryx2/issues/1087)
* Use `IOX2_SERVICE_NAME_LENGTH` in `ServiceName::to_string()`
    [#1095](https://github.com/eclipse-iceoryx/iceoryx2/issues/1095)
* Fix QNX cross compilation
    [#1116](https://github.com/eclipse-iceoryx/iceoryx2/issues/1116)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Decoupled tunnel implementation from tunelling mechanism
    [#845](https://github.com/eclipse-iceoryx/iceoryx2/issues/845)
* Explicitly use components from `core` and `alloc` in all Rust code
    [#865](https://github.com/eclipse-iceoryx/iceoryx2/issues/865)
* Updated all dependencies and increased MSRV to 1.83
    [#1105](https://github.com/eclipse-iceoryx/iceoryx2/issues/1105)
* Remove pre-compiled `noop.exe` used for testing command exeuction on Windows
    [#1133](https://github.com/eclipse-iceoryx/iceoryx2/issues/1133)
* Enable -Wconversion warning for the C and C++ code
    [#956](https://github.com/eclipse-iceoryx/iceoryx2/issues/956)
* Add thread sanitizer
    [#957](https://github.com/eclipse-iceoryx/iceoryx2/issues/957)

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### API Breaking Changes

1. **Rust:** Replaced the `FixedSizeVec` with the `StaticVec`

   ```rust
   // old
   use iceoryx2_bb_container::vec::FixedSizeVec;
   const VEC_CAPACITY: usize = 1234;
   let my_vec = FixedSizeVec::<MyType, VEC_CAPACITY>::new();

   // new
   use iceoryx2_bb_container::vector::*;
   const VEC_CAPACITY: usize = 1234;
   let my_vec = StaticVec::<MyType, VEC_CAPACITY>::new();
   ```

2. **Rust:** Replaced `Vec` with the `PolymorphicVec`

    ```rust
   // old
   use iceoryx2_bb_container::vec::Vec;
   const VEC_CAPACITY: usize = 1234;
   let my_vec = Vec::<MyType>::new();

   // new
   use iceoryx2_bb_container::vector::*;
   let my_stateful_allocator = acquire_allocator();
   let vec_capacity: usize = 1234;
   let my_vec = PolymorphicVec::<MyType>::new(my_stateful_allocator, vec_capacity)?;
    ```

3. **Rust:** Replaced the `FixedSizeByteString` with the `StaticString`

   ```rust
   // old
   use iceoryx2_bb_container::byte_string::FixedSizeString;
   const CAPACITY: usize = 1234;
   let my_str = FixedSizeByteString::<CAPACITY>::new();

   // new
   use iceoryx2_bb_container::string::*;
   const CAPACITY: usize = 1234;
   let my_str = StaticString::<CAPACITY>::new();
   ```

4. **C++:** Remove `operator*` and `operator->` from `ActiveRequest`,
   `PendingResponse`, `RequestMut`, `RequestMutUninit`, `Response`,
   `ResponseMut`, `Sample`, `SampleMut`, `SampleMutUninit` since these can
   easily lead to confusion and bugs when used in combination with `optional`
   or `expected`. See `sample.has_value()` and `sample->has_value()` that work
   on different objects.

   ```cxx
   // old
   auto sample = publisher.loan().expect("");
   sample->some_member = 123;

   // new
   auto sample = publisher.loan().expect("");
   sample.payload_mut().some_member = 123;
   ```

   ```cxx
   // old
   auto sample = publisher.loan().expect("");
   *sample = 123;
   std::cout << *sample << std::endl;

   // new
   auto sample = publisher.loan().expect("");
   sample.payload_mut() = 123;
   std::cout << sample.payload() << std::endl;
   ```

5. **Rust:** Changed the signature for Tunnel creation to take a concrete
   backend implementation

   ```rust
   // old
   let zenoh_config = zenoh::Config::default(); // coupled to zenoh
   let tunnel_config = iceoryx2_tunnel::TunnelConfig::default();
   let iceoryx_config = iceoryx2::config::Config::default();

   let mut tunnel =
       Tunnel::<Service>::create(&tunnel_config, &iceoryx_config, &zenoh_config).unwrap();

   // new
   let backend_config = Backend::Config::default();
   let tunnel_config = iceoryx2_tunnel::Config::default();
   let iceoryx_config = iceoryx2::config::Config::default();

   let mut tunnel =
       Tunnel::<Service, Backend>::create(&tunnel_config, &iceoryx_config, &backend_config).unwrap();
   ```
