# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->
* Enable Bazel `bzlmod` support for iceoryx2 builds
  [#355](https://github.com/eclipse-iceoryx/iceoryx2/issues/355)
* Android proof of concept with `local` communication
  [#416](https://github.com/eclipse-iceoryx/iceoryx2/issues/416)
* C, C++, and Python language bindings for blackboard
  [#817](https://github.com/eclipse-iceoryx/iceoryx2/issues/817)
* `iox2 config explain` cli command for config descriptions
  [#832](https://github.com/eclipse-iceoryx/iceoryx2/issues/832)
* Add traits to facilitate implementation of custom tunnelling mechanisms
  [#845](https://github.com/eclipse-iceoryx/iceoryx2/issues/845)
* Add support for `no_std` builds that can be enabled by disabling the new
  `std` feature when building `iceoryx2`
  [#865](https://github.com/eclipse-iceoryx/iceoryx2/issues/865)
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
* Support `unions` with `ZeroCopySend`
  [#1144](https://github.com/eclipse-iceoryx/iceoryx2/issues/1144)
* Add option to provide custom `iceoryx2-pal-configuration`
  [#1176](https://github.com/eclipse-iceoryx/iceoryx2/issues/1176)
* Add option to provide custom `iceoryx2-pal-posix`
  [#1176](https://github.com/eclipse-iceoryx/iceoryx2/issues/1176)
* Add shared memory variant based on files
  [#1223](https://github.com/eclipse-iceoryx/iceoryx2/issues/1223)
* Add socket directory configuration in platform
  [#1232](https://github.com/eclipse-iceoryx/iceoryx2/issues/1232)
* Replace legacy types in public API with iceoryx2 counterparts
  [#1234](https://github.com/eclipse-iceoryx/iceoryx2/issues/1234)

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
* Fix large server connection and data segment size
    [#1130](https://github.com/eclipse-iceoryx/iceoryx2/issues/1130)
* `ScopeGuard` check if `on_drop` is set before calling it
    [#1171](https://github.com/eclipse-iceoryx/iceoryx2/issues/1171)
* Fix C binding linker error on QNX
    [#1174](https://github.com/eclipse-iceoryx/iceoryx2/issues/1116)
* Fix panic during cleanup
    [#1198](https://github.com/eclipse-iceoryx/iceoryx2/issues/1198)
* Update urllib3 dependency to 2.6.0 (security issue in 2.5.0)
    [#1228](https://github.com/eclipse-iceoryx/iceoryx2/issues/1228)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Integrate the iceoryx_hoofs subset directly into the iceoryx2 repository
    [#301](https://github.com/eclipse-iceoryx/iceoryx2/issues/301)
* Decoupled tunnel implementation from tunelling mechanism
    [#845](https://github.com/eclipse-iceoryx/iceoryx2/issues/845)
* Factored out platform-specific build logic from common logic
    [#865](https://github.com/eclipse-iceoryx/iceoryx2/issues/865)
* Explicitly use components from `core` and `alloc` in all Rust code
    [#865](https://github.com/eclipse-iceoryx/iceoryx2/issues/865)
* Enable -Wconversion warning for the C and C++ code
    [#956](https://github.com/eclipse-iceoryx/iceoryx2/issues/956)
* Updated all dependencies and increased MSRV to 1.83
    [#1105](https://github.com/eclipse-iceoryx/iceoryx2/issues/1105)
* Remove pre-compiled `noop.exe` used for testing command exeuction on Windows
    [#1133](https://github.com/eclipse-iceoryx/iceoryx2/issues/1133)
* Support C++14 for the C++ Bindings
    [#1167](https://github.com/eclipse-iceoryx/iceoryx2/issues/1167)

### Workflow

1. **iceoryx_hoofs** dependency

The `iceoryx_hoofs` dependency was removed by importing the relevant files to
the iceoryx2 repository. This simplifies the build process makes it trivial to
add iceoryx2 specific features to the base lib.

The files from the `iceoryx_hoofs` subset are available via the `iceoryx2-bb-cxx`
CMake package.

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add `list_keys()` to list all keys stored in the blackboard,
  `EntryHandle::is_up_to_date()` to check for value updates
  [#1189](https://github.com/eclipse-iceoryx/iceoryx2/issues/1189)

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

6. Removed the `cdr` serializer from `iceoryx2-cal`, it is recommended to
   switch to the `postcard` serializer in its place

7. Merged `iox2/semantic_string.hpp` with imported `iox2/bb/semantic_string.hpp`
   from `iceoryx_hoofs`

   With this merge, the `SemanticStringError` moved from the `iox2` namespace
   into the `iox2::bb` namespace.

   ```cpp
   // old
   #include "iox2/semantic_string.hpp"
   // ...
   auto foo() -> expected<void, iox2::SemanticStringError>

   // new
   #include "iox2/bb/semantic_string.hpp"
   // ...
   auto foo() -> expected<void, iox2::bb::SemanticStringError>

8. **Rust:** The blackboard's `EntryValueUninit::write()` has been extended so
   that it also updates the entry and was renamed to `update_with_copy()`;
   `EntryValue` was removed.

   ```rust
   // old
   let entry_value = entry_value_uninit.write(123);
   let entry_handle_mut = entry_value.update();
   
   // new
   let entry_handle_mut = entry_value_uninit.update_with_copy(123);
   ```

8. Replace `iox::optional` from `iceoryx_hoofs` with `iox2::container::Optional`

  The new `Optional` in iceoryx2 has a reduced API compared to the one from
  `iceroyx_hoofs`. The functional interface, which deviated from the STL was
  removed.

  ```cpp
  // old
  ret_val.and_then([](auto& val) { /* do something with val */ })
         .or_else([]() { /* do something else */ });

  // new
  if (ret_val.has_value()) {
    // do something with ret_val.value()
  } else {
    // do something else
  }

  // old
  auto val = ret_val.expect("There should be a value");

  // new
  if (!ret_val.has_value()) {
    // error handling or terminate
  }
  auto val = ret_val.value();
  ```
