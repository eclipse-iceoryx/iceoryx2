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

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove default implementation of `ZeroCopySend` from `Option` and `Duration`
  [#1312](https://github.com/eclipse-iceoryx/iceoryx2/issues/1312)
* Bump wheel from 0.45.1 to 0.46.3 in /iceoryx2-ffi/python
  [#1316](https://github.com/eclipse-iceoryx/iceoryx2/issues/1316)
* Fix Python type translation for integer types (32-bit)
  [#1333](https://github.com/eclipse-iceoryx/iceoryx2/issues/1333)
* Fix GCC 9 build failure
  [#1342](https://github.com/eclipse-iceoryx/iceoryx2/issues/1342)
* Bump cryptography from 45.0.7 to 46.0.5 in /iceoryx2-ffi/python
  [#1316](https://github.com/eclipse-iceoryx/iceoryx2/issues/1348)
* Fix memory order in queue guards and index queue
  [#1356](https://github.com/eclipse-iceoryx/iceoryx2/issues/1356)
* Bump shellexpand from 3.1.1 to 3.1.2 in Rust and Bazel
  [#1365](https://github.com/eclipse-iceoryx/iceoryx2/issues/1365)
* Resources cannot always be cleaned up with `dev_permissions` feature flag
  [#1365](https://github.com/eclipse-iceoryx/iceoryx2/issues/1370)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->
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
