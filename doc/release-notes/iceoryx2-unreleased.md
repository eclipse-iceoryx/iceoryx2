# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Allow `dyn` types as `WaitSet` attachments
  [#1285](https://github.com/eclipse-iceoryx/iceoryx2/issues/1285)
* Propagate user headers in publish-subscribe samples in the reference tunnel
  implementation
  [#1289](https://github.com/eclipse-iceoryx/iceoryx2/issues/1289)
* Add source `NodeId` to request and response header
  [#1308](https://github.com/eclipse-iceoryx/iceoryx2/issues/1308)
* Introduce `RelocatableOption` and `RelocatableDuration` which are
  `ZeroCopySend`
  [#1312](https://github.com/eclipse-iceoryx/iceoryx2/issues/1312)
* Enable users to pull in iceoryx2 as a Bazel module/dependency
  [#1263](https://github.com/eclipse-iceoryx/iceoryx2/issues/1263)
* Add missing C++ APIs to access messaging pattern specific static config
  [#1353](https://github.com/eclipse-iceoryx/iceoryx2/issues/1353)
* Implement `core::error::Error` for `bb::posix` error enums
  [#1362](https://github.com/eclipse-iceoryx/iceoryx2/issues/1362)
* Add `thread_scope` as `std::thread::scope` counterpart
  [#1373](https://github.com/eclipse-iceoryx/iceoryx2/issues/1373)
* Add minimal Python event-based communication example and end-to-end test
  [#1376](https://github.com/eclipse-iceoryx/iceoryx2/issues/1376)
* Add `iox2 service hz` command with rolling-rate statistics and timeout support
  [#1383](https://github.com/eclipse-iceoryx/iceoryx2/issues/1383)
* Release Python GIL (detach thread from python runtime) in blocking functions
  like `listener.blocking_wait_one()`
  [#1421](https://github.com/eclipse-iceoryx/iceoryx2/issues/1421)
* Implement C++20 'source_location' based on compiler source location builtins
  and use it for better `Expected` and `Optional` messages on contract violations
  [#1496](https://github.com/eclipse-iceoryx/iceoryx2/issues/1496)
* Enable override of preallocated data chunks for sender ports
  [#1551](https://github.com/eclipse-iceoryx/iceoryx2/issues/1551)
* Add `Zeroable` and `PlainOldDataWithoutPadding` derive macros that generate
  the trait impl and enforce all invariants (`#[repr(C)]`, no padding, field
  bounds) at compile time
  [#1547](https://github.com/eclipse-iceoryx/iceoryx2/issues/1547)
* Enhance degradation handler and add unable to deliver handler
  [#1573](https://github.com/eclipse-iceoryx/iceoryx2/issues/1573)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Enable all tests for 32 bit after `#1567` fixed the unaligned pointer access
  [#262](https://github.com/eclipse-iceoryx/iceoryx2/issues/262)
* Publisher, Client, Server no longer deadlocks with
  UnableToDeliveryStrategy::Block when the other side disconnects
  [#314](https://github.com/eclipse-iceoryx/iceoryx2/issues/314)
* Update `set_log_level_X` docs and mention that iceoryx2 filters
  even for external frameworks
  [#620](https://github.com/eclipse-iceoryx/iceoryx2/issues/620)
* Remove timed and blocking send from `UnixDatagramSender` blocking block;
  these calls do not work on all supported platforms and were only used in
  the tests; the try send methods are used instead
  [#698](https://github.com/eclipse-iceoryx/iceoryx2/issues/698)
* Map the sender datasegment always as read-only
  [#906](https://github.com/eclipse-iceoryx/iceoryx2/issues/906)
* Remove default implementation of `ZeroCopySend` from `Option` and `Duration`
  [#1312](https://github.com/eclipse-iceoryx/iceoryx2/issues/1312)
* Bump wheel from 0.45.1 to 0.46.3 in /iceoryx2-ffi/python
  [#1316](https://github.com/eclipse-iceoryx/iceoryx2/issues/1316)
* Fix Python type translation for integer types (32-bit)
  [#1333](https://github.com/eclipse-iceoryx/iceoryx2/issues/1333)
* Fix GCC 9 build failure
  [#1342](https://github.com/eclipse-iceoryx/iceoryx2/issues/1342)
* Bump cryptography from 45.0.7 to 46.0.5 in /iceoryx2-ffi/python
  [#1348](https://github.com/eclipse-iceoryx/iceoryx2/issues/1348)
* Fix memory order in queue guards and index queue
  [#1356](https://github.com/eclipse-iceoryx/iceoryx2/issues/1356)
* Bump shellexpand from 3.1.1 to 3.1.2 in Rust and Bazel
  [#1365](https://github.com/eclipse-iceoryx/iceoryx2/issues/1365)
* Feature flags `tracing` and `log` were broken because `std` unconditionally
  enabled `console`; `console`, `log` and `tracing` are now exposed as top-level
  feature aliases in the `iceoryx2` crate
  [#1369](https://github.com/eclipse-iceoryx/iceoryx2/issues/1369)
* Resources cannot always be cleaned up with `dev_permissions` feature flag
  [#1370](https://github.com/eclipse-iceoryx/iceoryx2/issues/1370)
* Add `update_connection` to Python bindings
  [#1380](https://github.com/eclipse-iceoryx/iceoryx2/issues/1380)
* Add `Config::setup_global_config_from_file` to C++ bindings
  [#1395](https://github.com/eclipse-iceoryx/iceoryx2/issues/1395)
* Fix pointer provenance in `RelocatablePtr`
  [#1405](https://github.com/eclipse-iceoryx/iceoryx2/issues/1405)
* Bump keccak from 0.1.5 to 0.1.6 in Rust and Bazel
  [#1416](https://github.com/eclipse-iceoryx/iceoryx2/issues/1416)
* Bump black formatter from 25.1.0 to 26.3.1 in /iceoryx2-ffi/python
  [#1431](https://github.com/eclipse-iceoryx/iceoryx2/issues/1431)
* Output log entries with single write in console logger
  [#1432](https://github.com/eclipse-iceoryx/iceoryx2/issues/1432)
* Bump lz4_flex from 0.11.3 to 0.11.6 in Rust and Bazel
  [#1444](https://github.com/eclipse-iceoryx/iceoryx2/issues/1444)
* Fix libc dependency version
  [#1447](https://github.com/eclipse-iceoryx/iceoryx2/issues/1447)
* Fix FreeBSD build
  [#1455](https://github.com/eclipse-iceoryx/iceoryx2/issues/1455)
* Fix cleanup of resizable data segments
  [#1463](https://github.com/eclipse-iceoryx/iceoryx2/issues/1463)
* Bump rustls-webpki from 0.103.8 to 0.103.10 in Rust and Bazel
  [#1471](https://github.com/eclipse-iceoryx/iceoryx2/issues/1471)
* Fix deadlock in POSIX barrier in macOS
  [#1474](https://github.com/eclipse-iceoryx/iceoryx2/issues/1474)
* Fix `SIGPIPE` in `local::Service` events triggered by the `socketpair`
  [#1477](https://github.com/eclipse-iceoryx/iceoryx2/issues/1463)
* Bump requests from 2.32.5 to 2.33.0 in iceoryx2-ffi/python
  [#1486](https://github.com/eclipse-iceoryx/iceoryx2/issues/1486)
* Bump cryptography from 46.0.5 to 46.0.6 in /iceoryx2-ffi/python
  [#1499](https://github.com/eclipse-iceoryx/iceoryx2/issues/1499)
* Make `UniqueSystemId` unique accross docker containers and pid namespaces
  by introducing `gethostpid()` in the platform.
  [#1518](https://github.com/eclipse-iceoryx/iceoryx2/issues/1518)
* Bump cryptography from 46.0.6 to 46.0.7 in /iceoryx2-ffi/python
  [#1515](https://github.com/eclipse-iceoryx/iceoryx2/issues/1515)
* Bump pytest from 8.4.1 to 9.0.3 in /iceoryx2-ffi/python
  [#1515](https://github.com/eclipse-iceoryx/iceoryx2/issues/1515)
* Bump rand from 0.8.5 to 0.8.6 in Rust and Bazel
  [#1515](https://github.com/eclipse-iceoryx/iceoryx2/issues/1515)
* Bump rustls-webpki from 0.103.10 to 0.103.13 in Rust and Bazel
  [#1515](https://github.com/eclipse-iceoryx/iceoryx2/issues/1515)
* Fix build error in `iceoryx2-pal-posix` with bindgen
  [#1560](https://github.com/eclipse-iceoryx/iceoryx2/issues/1560)
* Fix alignment of the `variant` alternatives
  [#1567](https://github.com/eclipse-iceoryx/iceoryx2/issues/1567)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->
* Remove clippy workaround
  [#223](https://github.com/eclipse-iceoryx/iceoryx2/issues/223)
* Remove rule suppression in clang-tidy
  [#280](https://github.com/eclipse-iceoryx/iceoryx2/issues/280)
* Remove support for Bazel Workspaces
  [#1263](https://github.com/eclipse-iceoryx/iceoryx2/issues/1263)
* Adjust test names to naming convention
  [#1273](https://github.com/eclipse-iceoryx/iceoryx2/issues/1273)
* Move character output abstraction into their own crate
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Move `iceoryx2-loggers` crate into building blocks as `iceoryx2-bb-loggers`
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Replace `lazy_static` dependency with `LazyLock` from `std` in `std` builds or
  a custom minimal spin-based implementation for `no_std` builds
  [#1321](https://github.com/eclipse-iceoryx/iceoryx2/issues/1321)
* Remove `auto` option from Bazel feature flags and align defaults with CMake
  [#1326](https://github.com/eclipse-iceoryx/iceoryx2/issues/1326)
* Remove `posix` feature and use `cfg` switch based on target instead
  [#1327](https://github.com/eclipse-iceoryx/iceoryx2/issues/1327)
* `CleanupState` implements `ZeroCopySend`
  [#1331](https://github.com/eclipse-iceoryx/iceoryx2/issues/1331)
* Ignore warnings from bindgen generated files with bazel build
  [#1345](https://github.com/eclipse-iceoryx/iceoryx2/issues/1345)
* Set Rust minimum required version (MSRV) to version 1.85.0
  [#1359](https://github.com/eclipse-iceoryx/iceoryx2/issues/1359)
* Upgrade Rust Edition to 2024
  [#1359](https://github.com/eclipse-iceoryx/iceoryx2/issues/1359)
* Use `libc` constants in linux platform instead of hardcoded values
  [#1388](https://github.com/eclipse-iceoryx/iceoryx2/issues/1388)
* Update `inventory` crate and remove `disallowed_types` suppression from tests
  [#1492](https://github.com/eclipse-iceoryx/iceoryx2/issues/1492)
* Rename `ServiceId` into `ServiceHash`
  [#1508](https://github.com/eclipse-iceoryx/iceoryx2/issues/1508)
* Rename `NodeId` into `UniqueNodeId`
  [#1534](https://github.com/eclipse-iceoryx/iceoryx2/issues/1534)
* Move conformance test macros into `iceoryx2-bb-testing-macros` and rename
  `#[conformance_test_module]` to `#[conformance_tests]`
  [#1540](https://github.com/eclipse-iceoryx/iceoryx2/issues/1540)
* Move tunnel crates into services architecture layer
  [#1552](https://github.com/eclipse-iceoryx/iceoryx2/issues/1552)

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add custom test framework that supports `no_std` testing
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Add `no_std` tests for `iceoryx2` and crates below it in the architecture
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Add CI check for `std` feature propagation
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Enable clippy for the whole workspace and all targets
  [#1355](https://github.com/eclipse-iceoryx/iceoryx2/issues/1355)
* Add `just` scripts for some common maintenance tasks
  [#1408](https://github.com/eclipse-iceoryx/iceoryx2/issues/1408)

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Removed `libc_platform` feature, platforms that support the crate `libc` will
  now automatically use it
  [#1374](https://github.com/eclipse-iceoryx/iceoryx2/issues/1374)

### API Breaking Changes

1. Loggers have been moved into `iceoryx2-bb-loggers` thus manually linking
   to them needs to be adjusted accordingly.

    ```rust
    // old
    extern crate iceoryx2_loggers;

    use iceoryx2_log::*;

    set_log_level(LogLevel::Info);
    info!("some log message")

    // new
    extern crate iceoryx2_bb_loggers;

    use iceoryx2_log::*;

    set_log_level(LogLevel::Info);
    info!("some log message")
    ```

1. Building with `libc` is now default on platforms that support it

    ```console
    # old
    cargo build --features iceoryx2/libc_platform

    # new
    cargo build
    ```

1. `ServiceId` was renamed to `ServiceHash`.

    ```rust
    // old
    use iceoryx2::*;

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .publish_subscribe::<TransmissionData>()
        .open_or_create()?;
    service.service_id(); // now service_hash()

    // new
   use iceoryx2::*;

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .publish_subscribe::<TransmissionData>()
        .open_or_create()?;
    service.service_hash();
    ```

1. `NodeId` was renamed to `UniqueNodeId`.

    ```rust
    // old
    use iceoryx2::*;

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let id: NodeId = node.id();

    // new
    use iceoryx2::*;

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let id: UniqueNodeId = node.id();
    ```

1. The `#[conformance_test_module]` macro has been renamed to
   `#[conformance_tests]`

   ```rust
   // old
    #[allow(clippy::module_inception)]
    #[conformance_test_module]
    pub mod my_module {
        // ...
    }

   // new
   #[allow(clippy::module_inception)]
   #[conformance_tests]
   pub mod my_module {
       // ...
   }
   ```

1. The crates for the `iceoryx2` tunnel have been moved into
   the `services` architecture layer and have been renamed
   accordingly.

    ```rust
    // old
    use iceoryx2_tunnel::{Config, Tunnel};

    // new
    use iceoryx2_services_tunnel::{Config, Tunnel};
    ```

1. The `UnableToDeliveryStrategy` enum tags are renamed to from `Block` and
   `DiscardSample` to `RetryUntilDelivered` and `DiscardData`.

    ```rust
    // old
    service
        .publisher_builder()
        .unable_to_deliver_strategy(UnableToDeliverStrategy::Block)
        .create()?;

    // new
    service
        .publisher_builder()
        .unable_to_deliver_strategy(UnableToDeliverStrategy::RetryUntilDelivered)
        .create()?;
    ```

1. The `DegradationCallback` was renamed to `DegradationHandler`, as well as its
   setters and the parameter changed from port IDs to `DegradationCause` and
   `DegradationInfo`

    ```rust
    /// old
    pub_sub_service
        .publisher_builder()
        .set_degradation_callback(|sender_port_id, receiver_port_id| /* ... */)
        .create()?
    pub_sub_service
        .subscriber_builder()
        .set_degradation_callback(|sender_port_id, receiver_port_id| /* ... */)
        .create()?

    request_response_service
        .client_builder()
        .set_request_degradation_callback(|sender_port_id, receiver_port_id| /* ... */)
        .set_response_degradation_callback(|sender_port_id, receiver_port_id| /* ... */)
        .create()?
    request_response_service
        .server_builder()
        .set_request_degradation_callback(|sender_port_id, receiver_port_id| /* ... */)
        .set_response_degradation_callback(|sender_port_id, receiver_port_id| /* ... */)
        .create()?

    /// new
    pub_sub_service
        .publisher_builder()
        .set_degradation_handler(|cause, info| /* ... */)
        .create()?
    pub_sub_service
        .subscriber_builder()
        .set_degradation_handler(|cause, info| /* ... */)
        .create()?

    request_response_service
        .client_builder()
        .set_request_degradation_handler(|cause, info| /* ... */)
        .set_response_degradation_handler(/* ... */)
        .create()?
    request_response_service
        .server_builder()
        .set_request_degradation_handler(|cause, info|/* ... */)
        .set_response_degradation_handler(|cause, info|/* ... */)
        .create()?
    ```

1. The `DegradationAction::Fail` was renamed to `DegradationAction::DegradeAndFail`.

    ```rust
    // old
    service
        .publisher_builder()
        .set_degradation_callback(|_, _| DegradationAction::Fail)
        .create()?;

    // new
    service
        .publisher_builder()
        .set_degradation_handler(|_, _| DegradationAction::DegradeAndFail)
        .create()?;
    ```
