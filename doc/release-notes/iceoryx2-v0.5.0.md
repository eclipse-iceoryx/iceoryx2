# iceoryx2 v0.5.0

## [v0.5.0](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.5.0)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.4.1...v0.5.0)

### Features

* C++ bindings for attributes [#264](https://github.com/eclipse-iceoryx/iceoryx2/issues/264)
* Add Event-Multiplexer `WaitSet` [#390](https://github.com/eclipse-iceoryx/iceoryx2/issues/390)
* Add `PeriodicTimer` into POSIX building blocks [#425](https://github.com/eclipse-iceoryx/iceoryx2/issues/425)
* Developer permissions for resources [#460](https://github.com/eclipse-iceoryx/iceoryx2/issues/460)
* Add `--send-copy` flag to Benchmark to consider mem operations [#483](https://github.com/eclipse-iceoryx/iceoryx2/issues/483)
* Support for slices in the C++ bindings [#490](https://github.com/eclipse-iceoryx/iceoryx2/issues/490)
* Add API to retrieve string description of error enums [$491](https://github.com/eclipse-iceoryx/iceoryx2/issues/491)
* Add relocatable `SlotMap` [#504](https://github.com/eclipse-iceoryx/iceoryx2/issues/504)
* Add `ResizableSharedMemory` [#497](https://github.com/eclipse-iceoryx/iceoryx2/issues/497)
* Make signal handling optional in `WaitSet` and `Node` [#528](https://github.com/eclipse-iceoryx/iceoryx2/issues/528)
* Support dynamic data with reallocation for publish-subscribe communication [#532](https://github.com/eclipse-iceoryx/iceoryx2/issues/532)
* Add benchmark for iceoryx2 queues [#535](https://github.com/eclipse-iceoryx/iceoryx2/issues/535)
* Add auto event mission for create, drop and dead notifiers [#550](https://github.com/eclipse-iceoryx/iceoryx2/issues/550)
* Introduce health monitoring example [#555](https://github.com/eclipse-iceoryx/iceoryx2/issues/555)
* Reuse existing cargo build with C and C++ bindings [#559](https://github.com/eclipse-iceoryx/iceoryx2/issues/559)

### Bugfixes

* Split SignalHandler signals to avoid infinite loops on SIGSEGV
  [#436](https://github.com/eclipse-iceoryx/iceoryx2/issues/436)
* Fix misleading warning related to default config file
  [#437](https://github.com/eclipse-iceoryx/iceoryx2/issues/437)
* Fix infinite loop triggering in `WaitSet`
  [#518](https://github.com/eclipse-iceoryx/iceoryx2/issues/518)
* Fix cmake build with iceoryx2 as submodule
  [#521](https://github.com/eclipse-iceoryx/iceoryx2/issues/521)

### Refactoring

* Rename `NodeEvent` into `WaitEvent` [#390](https://github.com/eclipse-iceoryx/iceoryx2/issues/390)
* Bazel support for the Rust crates [#349](https://github.com/eclipse-iceoryx/iceoryx2/issues/349)
* Remove ACL dependency [#457](https://github.com/eclipse-iceoryx/iceoryx2/issues/457)
* Remove `max_slice_len` publisher builder option for non-slice types [#496](https://github.com/eclipse-iceoryx/iceoryx2/issues/496)
* Publish Subscribe Header contains number of elements contained in a `Sample` [#498](https://github.com/eclipse-iceoryx/iceoryx2/issues/498)

### New API features

* APIs to support slices in the C/C++ bindings [#490](https://github.com/eclipse-iceoryx/iceoryx2/issues/490)
* Rename `iox2_publisher_loan` to `iox2_publisher_loan_slice_uninit` [#490](https://github.com/eclipse-iceoryx/iceoryx2/issues/490)
    1. C always loans slices, for a single element, specify the
       `number_of_elements` to be 1
* Add APIs to C/C++ bindings to get string representation of error enum [#491](https://github.com/eclipse-iceoryx/iceoryx2/issues/491)
    1. C API: `iox2_{error_enum_name}_string(enum_value)`
    2. C++ API: `iox::into<const char*>(enum_value)`
* APIs to retrieve the value of `UniquePortIds` from the C/C++ bindings [#500](https://github.com/eclipse-iceoryx/iceoryx2/issues/500)

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

3. Renamed `max_slice_len()` into `initial_max_slice_len()`.

   ```rust
   // old
   let publisher = service
        .publisher_builder()
        .max_slice_len(16)
        .create()?;

   // new
   let publisher = service
        .publisher_builder()
        .initial_max_slice_len(16)
        .create()?;
   ```
