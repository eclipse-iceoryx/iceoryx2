<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v0.9.2

## [v0.9.2](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.9.2)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.9.1...v0.9.2)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1641](https://github.com/eclipse-iceoryx/iceoryx2/issues/1641) Deliver with Backpressure::Retry when receiver is disconnected until the buffer is full.
* [#1690](https://github.com/eclipse-iceoryx/iceoryx2/issues/1690) Fix dependencies in iceoryx2-bb-loggers to re-enable `bazel query`.
* [#1699](https://github.com/eclipse-iceoryx/iceoryx2/issues/1699) Enforce synchronization with compare exchange in UnrestrictedAtomic, RobustUniqueIndexSet and Container.
* [#1724](https://github.com/eclipse-iceoryx/iceoryx2/issues/1724) Fix chunk leak when offset cannot be translated to dynamic data segment.
* [#1733](https://github.com/eclipse-iceoryx/iceoryx2/issues/1733) `iox2 service replay` sends full payload for dynamic data.

<!-- markdownlint-enable MD013 -->
