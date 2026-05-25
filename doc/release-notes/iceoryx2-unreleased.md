<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1654](https://github.com/eclipse-iceoryx/iceoryx2/issues/1654) Introduce `AdaptiveWaitBehavior`

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1650](https://github.com/eclipse-iceoryx/iceoryx2/issues/1650) Fix 'NodeCreationFailure::InternalError' on concurrent node creation
* [#1660](https://github.com/eclipse-iceoryx/iceoryx2/issues/1660) Rework process alive detection to avoid false negatives to enable communication across docker containers
* [#1670](https://github.com/eclipse-iceoryx/iceoryx2/issues/1670) Fix SIGBUS on cleanup with write-only shared memory with root
* [#1675](https://github.com/eclipse-iceoryx/iceoryx2/issues/1675) Make 'NamedConcept::remove_cfg()' always fail when removal fails
* [#1681](https://github.com/eclipse-iceoryx/iceoryx2/issues/1681) Deactivate 'cleanup_dead_nodes_on_open' for 'send_dead_node_signal'
* [#1682](https://github.com/eclipse-iceoryx/iceoryx2/issues/1682) Use 'AdaptiveWait' with 'FixedTicks' when the filesystem is accessed

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1651](https://github.com/eclipse-iceoryx/iceoryx2/issues/1651) Make the names of the test binaries relatable to the crates

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

### API Breaking Changes

1. Example

   ```rust
   // old
   let fuu = hello().is_it_me_you_re_looking_for()

   // new
   let fuu = hypnotoad().all_glory_to_the_hypnotoad()
   ```

<!-- markdownlint-enable MD013 -->
