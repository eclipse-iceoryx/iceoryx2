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
* Developer permissions for resources [#460](https://github.com/eclipse-iceoryx/iceoryx2/issues/460)
* Add `--send-copy` flag to Benchmark to consider mem operations [#483](https://github.com/eclipse-iceoryx/iceoryx2/issues/483)
* Support for slices in the C++ bindings [#490](https://github.com/eclipse-iceoryx/iceoryx2/issues/490)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Split SignalHandler signals to avoid infinite loops on SIGSEGV
  [#436](https://github.com/eclipse-iceoryx/iceoryx2/issues/436)
* Fix misleading warning related to default config file
  [#437](https://github.com/eclipse-iceoryx/iceoryx2/issues/437)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Rename `NodeEvent` into `WaitEvent` [#390](https://github.com/eclipse-iceoryx/iceoryx2/issues/390)
* Bazel support for the Rust crates [#349](https://github.com/eclipse-iceoryx/iceoryx2/issues/349)
* Remove ACL dependency [#457](https://github.com/eclipse-iceoryx/iceoryx2/issues/457)
* Publish Subscribe Header contains number of elements contained in a `Sample` [#498](https://github.com/eclipse-iceoryx/iceoryx2/issues/498)

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

* APIs to support slices in the C++ bindings [#490](https://github.com/eclipse-iceoryx/iceoryx2/issues/490)
* Rename `iox2_publisher_loan` to `iox2_publisher_loan_slice_uninit` [#490](https://github.com/eclipse-iceoryx/iceoryx2/issues/490)
    1. C always loans slices, for a single element, specify the
       `number_of_elements` to be 1

### API Breaking Changes

1. Removed `NodeEvent`. `Node::wait` returns now `Result<(), NodeWaitFailure>`

   ```rust
   // old
   while node.wait(CYCLE_TIME) != NodeEvent::TerminationRequest {
    // ...
   }

   // new
   while node.wait(CYCLE_TIME).is_ok() {
    // ...
   }
   ```

2. Removed `payload_type_layout()` from `publish_subscribe::Header`.
