<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v0.9.1

## [v0.9.1](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.9.1)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.9.0...v0.9.1)

### Features

* [#1654](https://github.com/eclipse-iceoryx/iceoryx2/issues/1654) Introduce `AdaptiveWaitBehavior`

### Bugfixes

* [#1650](https://github.com/eclipse-iceoryx/iceoryx2/issues/1650) Fix 'NodeCreationFailure::InternalError' on concurrent node creation
* [#1660](https://github.com/eclipse-iceoryx/iceoryx2/issues/1660) Rework process alive detection to avoid false negatives to enable communication across docker containers
* [#1670](https://github.com/eclipse-iceoryx/iceoryx2/issues/1670) Fix SIGBUS on cleanup with write-only shared memory with root
* [#1675](https://github.com/eclipse-iceoryx/iceoryx2/issues/1675) Make 'NamedConcept::remove_cfg()' always fail when removal fails
* [#1681](https://github.com/eclipse-iceoryx/iceoryx2/issues/1681) Deactivate 'cleanup_dead_nodes_on_open' for 'send_dead_node_signal'
* [#1682](https://github.com/eclipse-iceoryx/iceoryx2/issues/1682) Use 'AdaptiveWait' with 'FixedTicks' when the filesystem is accessed

### Refactoring

* [#1651](https://github.com/eclipse-iceoryx/iceoryx2/issues/1651) Make the names of the test binaries relatable to the crates

<!-- markdownlint-enable MD013 -->
