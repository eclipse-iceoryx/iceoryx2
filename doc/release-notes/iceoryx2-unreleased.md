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
* Refactor to enable users to pull in Iceoryx2 as a Bazel module/dependency
  [#1263](https://github.com/eclipse-iceoryx/iceoryx2/issues/1263)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove default implementation of `ZeroCopySend` from `Option` and `Duration`
  [#1312](https://github.com/eclipse-iceoryx/iceoryx2/issues/1312)
* Bump wheel from 0.45.1 to 0.46.3 in /iceoryx2-ffi/python
  [#1316](https://github.com/eclipse-iceoryx/iceoryx2/issues/1316)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->
* Adjust test names to naming convention
  [#1273](https://github.com/eclipse-iceoryx/iceoryx2/issues/1273)
* Move character output abstraction into their own crate
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Move `iceoryx2-loggers` crate into building blocks as `iceoryx2-bb-loggers`
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Replace `lazy_static` dependency with `LazyLock` from `std` in `std` builds or
  a custom minimal spin-based implementation for `no_std` builds
  [#1321](https://github.com/eclipse-iceoryx/iceoryx2/issues/1321)

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
