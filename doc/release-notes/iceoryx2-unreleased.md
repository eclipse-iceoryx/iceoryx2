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

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Fix feature propagation for `libc_platform`
  [#1282](https://github.com/eclipse-iceoryx/iceoryx2/issues/1282)
* Update urllib3 dependency to 2.6.3 (security issue in 2.6.0)
  [#1290](https://github.com/eclipse-iceoryx/iceoryx2/issues/1290)
* Fix race condition in node `RegisteredService` struct
  [#1293](https://github.com/eclipse-iceoryx/iceoryx2/issues/1293)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Move character output abstraction into building blocks as `iceoryx2-bb-print`
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)
* Move `iceoryx2-loggers` crate into building blocks as `iceoryx2-bb-loggers`
  [#1300](https://github.com/eclipse-iceoryx/iceoryx2/issues/1300)

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
