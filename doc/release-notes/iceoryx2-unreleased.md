# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add Event-Multiplexer `WaitSet` [#390](https://github.com/eclipse-iceoryx/iceoryx2/issues/390)
* Add `PeriodicTimer` into POSIX building blocks [#425](https://github.com/eclipse-iceoryx/iceoryx2/issues/425)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Fix misleading warning related to default config file
  [#437](https://github.com/eclipse-iceoryx/iceoryx2/issues/437)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Rename `NodeEvent` into `WaitEvent` [#390](https://github.com/eclipse-iceoryx/iceoryx2/issues/390)

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

1. Renamed `NodeEvent` into `WaitEvent`

   ```rust
   // old
   while node.wait(CYCLE_TIME) != NodeEvent::TerminationRequest {
    // ...
   }

   // new
   while node.wait(CYCLE_TIME) != WaitEvent::TerminationRequest {
    // ...
   }
   ```
