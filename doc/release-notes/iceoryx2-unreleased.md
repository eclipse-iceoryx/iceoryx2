<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text
* [#2001](https://github.com/eclipse-iceoryx/iceoryx2/issues/2001) Add listener traversal and single-listener notification APIs to C, C++, and Python bindings
* [#2011](https://github.com/eclipse-iceoryx/iceoryx2/issues/2011) Add a publish-subscribe latency benchmark with a FlatBuffers payload

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text
* [#2018](https://github.com/eclipse-iceoryx/iceoryx2/issues/2018) Return `InvalidListenerKey` instead of panicking when notifying with a listener key whose index exceeds the service capacity

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#2010](https://github.com/eclipse-iceoryx/iceoryx2/issues/2010) Receive bytes from a tunnel directly into loaned samples

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text
* [#2001](https://github.com/eclipse-iceoryx/iceoryx2/issues/2001) Add retainable listener keys, callback-scoped mono notifiers, and listener names for filtering

### API Breaking Changes

1. Example

   ```rust
   // old
   let fuu = hello().is_it_me_you_re_looking_for()

   // new
   let fuu = hypnotoad().all_glory_to_the_hypnotoad()
   ```

<!-- markdownlint-enable MD013 -->
