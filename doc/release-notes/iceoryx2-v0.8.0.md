# iceoryx2 v0.8.0

## [v0.8.0](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.8.0)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.7.0...v0.8.0)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->
* Separate log crate into front-end API and backend logger implementation crates
  [#154](https://github.com/eclipse-iceoryx/iceoryx2/issues/154)
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
* Add platform support for QNX 8
  [#853](https://github.com/eclipse-iceoryx/iceoryx2/issues/853)
* Add logger backend that uses the POSIX APIs instead of `std` to log to the
  console
  [#853](https://github.com/eclipse-iceoryx/iceoryx2/issues/853)
* Add support for `no_std` builds that can be enabled by disabling the new
  `std` feature when building `iceoryx2`
  [#865](https://github.com/eclipse-iceoryx/iceoryx2/issues/865)
* Add a C++ string container type with fixed compile-time capacity
  [#938](https://github.com/eclipse-iceoryx/iceoryx2/issues/938)
* Add new C++ `Expected` vocabulary data type
  [#940](https://github.com/eclipse-iceoryx/iceoryx2/issues/940)
* Add a C++ vector container type with fixed compile-time capacity
  [#951](https://github.com/eclipse-iceoryx/iceoryx2/issues/951)
* Use `epoll` instead of `select` for the `WaitSet` on Linux
  [#961](https://github.com/eclipse-iceoryx/iceoryx2/issues/961)
* Configuration mechanism to for replacing vocabulary types in iceoryx2-bb-cxx
  with STL types
  [#971](https://github.com/eclipse-iceoryx/iceoryx2/issues/971)
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
* Configuration mechanism to for replacing vocabulary types in iceoryx2-bb-cxx
  with custom implementations
  [#1250](https://github.com/eclipse-iceoryx/iceoryx2/issues/1250)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Print new line after CLI output to prevent '%' from being inserted by terminal
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)
* Print help for positional arguments in CLI
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)
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
* Fix that `dev_permissions` are not applied to all resources
    [#1188](https://github.com/eclipse-iceoryx/iceoryx2/issues/1188)
* Fix panic during cleanup
    [#1198](https://github.com/eclipse-iceoryx/iceoryx2/issues/1198)
* Update urllib3 dependency to 2.6.0 (security issue in 2.5.0)
    [#1228](https://github.com/eclipse-iceoryx/iceoryx2/issues/1228)
* Fix incomplete cleanup of file lock based monitoring concept
    [#1252](https://github.com/eclipse-iceoryx/iceoryx2/issues/1252)

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
* Remove dependency on `iceoryx2-pal-concurrency-sync` in upper-level crates
    [#1202](https://github.com/eclipse-iceoryx/iceoryx2/issues/1202)
* Replace legacy types in public API with iceoryx2 counterparts
    [#1234](https://github.com/eclipse-iceoryx/iceoryx2/issues/1234)

### Workflow

* Removed `iceoryx2_hoofs` dependency by importing relevant files into
  a new `iceoryx2-bb-cxx` CMake package to simplify the build process.
    [#301](https://github.com/eclipse-iceoryx/iceoryx2/issues/301)
* Add end-to-end tests for `iceoryx2-cli`
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add `list_keys()` to list all keys stored in the blackboard,
  `EntryHandle::is_up_to_date()` to check for value updates
    [#1189](https://github.com/eclipse-iceoryx/iceoryx2/issues/1189)
* Add option to force overwrite configuration with `iox2 config generate`
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)

### API Breaking Changes

1. **Rust:** Split logger frontend and backend, requiring the crate
   `iceoryx2_loggers` to be linked to all binaries so that the logger
   backend can be used by the frontend.

   ```rust
   // old
   use iceoryx2_bb_log::*;

   set_log_level(LogLevel::Info);
   info!("some log message")

   // new
   extern crate iceoryx2_loggers;

   use iceoryx2_log::*;

   set_log_level(LogLevel::Info);
   info!("some log message")
   ```

   Binary crates must also include `iceoryx2_loggers` as a dependency and the
   default logger must be specified via feature flags. If no
   feature flag is enabled, the logs are discarded. Details are documented
   in the `iceoryx2-loggers` crate.

    ```toml
    iceoryx2-loggers = { version = "0.7.0", features = ["std", "console"] }
    ```

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

1. **Rust:** Replaced `Vec` with the `PolymorphicVec`

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

1. **Rust:** Replaced the `FixedSizeByteString` with the `StaticString`

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

1. **C++:** Remove `operator*` and `operator->` from `ActiveRequest`,
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

1. **Rust:** Changed the signature for Tunnel creation to take a concrete
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

1. Removed the `cdr` serializer from `iceoryx2-cal`, it is recommended to
   switch to the `postcard` serializer in its place

1. Merged `iox2/semantic_string.hpp` with imported `iox2/bb/semantic_string.hpp`
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

1. **Rust:** The blackboard's `EntryValueUninit::write()` has been extended so
   that it also updates the entry and was renamed to `update_with_copy()`;
   `EntryValue` was removed.

   ```rust
   // old
   let entry_value = entry_value_uninit.write(123);
   let entry_handle_mut = entry_value.update();

   // new
   let entry_handle_mut = entry_value_uninit.update_with_copy(123);
   ```

1. **C++:** Replace `iox::optional` from `iceoryx_hoofs` with
   `iox2::bb::Optional`

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

1. Replace `iox::expected` from `iceoryx_hoofs` with `iox2::bb::Expected`

   The new `Expected` in iceoryx2 has a reduced API compared to the one from
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
     // do something with ret_val.error()
   }

   // old
   auto val = ret_val.expect("There should be a value");

   // new
   if (!ret_val.has_value()) {
     // error handling based on ret_val.error() or terminate
   }
   auto val = ret_val.value();
   ```

1. **C++:** Replace `iox::string` from `iceoryx_hoofs` with
   `iox2::bb::StaticString`.

   ```cpp
   // old
   auto str = iox::string<10>("hello");
   std::cout << str.c_str() << std::endl;

   // new
   auto str = iox2::bb::StaticString<10>::from_utf8("hello");
   if (str.has_value()) {
       std::cout << str->unchecked_access().c_str() << std::endl;
   }
   ```

1. **C++** The `Attribute::Key` and `Attribute::Value` types switched from
   `iox::string` to `iox2::bb::StaticString`

   ```cpp
   // old
   auto attribute_specifier = AttributeSpecifier().define("my_key", "my_value");

   // new
   auto my_key = Attribute::Key::from_utf8("my_key");
   auto my_value = Attribute::Value::from_utf8("my_value");
   if (!my_key.has_value() || !my_value.has_value()) {
       // do error handling
   }
   auto attribute_specifier = AttributeSpecifier().define(*my_key, *my_value);
   ```


1. **C++** Moved classes from `iceoryx_hoofs` to `iceoryx2` and adjusted
   namespaces and naming convention

   This is a non-exhaustive list of the changes. Please check the API
   documentation for more details.

   ```cpp
   // old
   #include "iox/duration.hpp"
   auto d = iox::units::Duration::fromMilliseconds(42);

   // new
   #include "iox2/bb/duration.hpp"
   auto d = iox2::bb::Duration::from_millis(42);


   // old
   #include "iox/into.hpp"
   auto foo = iox::into<Foo>(bar);

   // new
   #include "iox2/bb/into.hpp"
   auto foo = iox2::bb::into<Foo>(bar);


   // old
   #include "iox/file_name.hpp"
   #include "iox/file_path.hpp"
   #include "iox/path.hpp"
   auto file_name = iox::FileName::create("foo.txt").expect("Creating file name");
   auto file_path = iox::FilePath::create("/home/hypnotoad/foo.txt").expect("Creating file path");
   auto path = iox::Path::create("/home/hypnotoad/").expect("Creating path");

   // new
   #include "iox2/bb/file_name.hpp"
   #include "iox2/bb/file_path.hpp"
   #include "iox2/bb/path.hpp"
   auto file_name_result = iox2::bb::FileName::create("foo.txt");
   if (!file_name_result.has_value()) {
       // do error handling
   }
   auto file_name = file_name_result.value();

   auto file_path_result = iox2::bb::FilePath::create("/home/hypnotoad/foo.txt");
   if (!file_path_result.has_value()) {
       // do error handling
   }
   auto file_path = file_path_result.value();

   auto path_result = iox2::bb::Path::create("/home/hypnotoad/");
   if (!path_result.has_value()) {
       // do error handling
   }
   auto path = path_result.value();


   // old
   #include "iox/slice.hpp
   auto s = iox::Slice<uint8_t>(dapa.ptr(), data.len());

   // new
   #include "iox2/bb/slice.hpp"
   auto s = iox2::bb::Slice<uint8_t>(dapa.ptr(), data.len());
   ```

1. Add summarized and detailed variants of `iox2 service discovery`

   ```console
   // old
   $ iox2 service discovery
   === Service Started (rate: 100ms) ===
   Added((
       service_id: ("4eacadf2695a3f4b2eb95485759246ce1a2aa906"),
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       messaging_pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
   ))
   Removed((
       service_id: ("4eacadf2695a3f4b2eb95485759246ce1a2aa906"),
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       messaging_pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
   ))

   // new
   $ iox2 service discovery
   Discovering Services (rate: 100ms)
   Added(PublishSubscribe("My/Funk/ServiceName"))
   Removed(PublishSubscribe("My/Funk/ServiceName"))

   $ iox2 service discovery --detailed
   Discovering Services (rate: 100ms)
   Added((
       service_id: "4eacadf2695a3f4b2eb95485759246ce1a2aa906",
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
       nodes: Some((
           num: 1,
           details: [
               (
                   state: Alive,
                   id: ("0000000034fcd3b8000013a8000135c1"),
                   pid: 79297,
                   executable: Some("publish_subscribe_subscriber"),
                   name: Some(""),
               ),
           ],
       )),
   ))
   Removed((
       service_id: "4eacadf2695a3f4b2eb95485759246ce1a2aa906",
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
       nodes: Some((
           num: 1,
           details: [
               (
                   state: Alive,
                   id: ("0000000034fcd3b8000013a8000135c1"),
                   pid: 79297,
                   executable: Some("publish_subscribe_subscriber"),
                   name: Some(""),
               ),
           ],
       )),
   ))
   ```
