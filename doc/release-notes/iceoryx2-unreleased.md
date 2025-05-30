# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Tunnel over zenoh for publish-subscribe and event services
  [#593](https://github.com/eclipse-iceoryx/iceoryx2/issues/593)
* `iox2 tunnel zenoh`
  [#593](https://github.com/eclipse-iceoryx/iceoryx2/issues/593)
* All port factories implement `Send`
  [#768](https://github.com/eclipse-iceoryx/iceoryx2/issues/768)
* `iox2 service listen` and `iox2 service notify`
  [#790](https://github.com/eclipse-iceoryx/iceoryx2/issues/790)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Fix segmentation fault in event multiplexing C example
    [#766](https://github.com/eclipse-iceoryx/iceoryx2/issues/766)
* Fix lifetime in ThreadGuardedStackBuilder
    [#770](https://github.com/eclipse-iceoryx/iceoryx2/issues/770)
* Fix config file load failure
    [#772](https://github.com/eclipse-iceoryx/iceoryx2/issues/772)
* Make `Directory::create` thread-safe
    [#778](https://github.com/eclipse-iceoryx/iceoryx2/issues/778)
* Make CLI generated global config file accessible to all users
    [#786](https://github.com/eclipse-iceoryx/iceoryx2/issues/786)
* Make `iox2 config show` print the available options
    [#788](https://github.com/eclipse-iceoryx/iceoryx2/issues/788)
* Fix bug where CLI was not displaying positional arguments in usage help
    [#796](https://github.com/eclipse-iceoryx/iceoryx2/issues/796)
* Fix startup race in `pthread_create` on mac os platform
    [#799](https://github.com/eclipse-iceoryx/iceoryx2/issues/799)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Implement `ZeroCopySend` for all system types
  [#732](https://github.com/eclipse-iceoryx/iceoryx2/issues/732)
* Remove trait re-exports from iceoryx2-bb-elementary
  [#757](https://github.com/eclipse-iceoryx/iceoryx2/issues/757)
* Make POSIX user- and group details optional
  [#780](https://github.com/eclipse-iceoryx/iceoryx2/issues/780)
* Add `recommended::Ipc` and `recommended::Local` to iceoryx2 concepts for
  to provide link the best implementation for the specific platform
  [#806](https://github.com/eclipse-iceoryx/iceoryx2/issues/806)

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

* Add `NodeId` to sample header (to prevent loopback in tunnels)
  [#593](https://github.com/eclipse-iceoryx/iceoryx2/issues/593)
* Add API to prevent self notification `Notifier::__internal_notify()`
  [#794](https://github.com/eclipse-iceoryx/iceoryx2/issues/794)
* Enable the usage of semaphore based events in C/C++
  [#795](https://github.com/eclipse-iceoryx/iceoryx2/issues/795)

### API Breaking Changes

1. Example

   ```rust
   // old
   let fuu = hello().is_it_me_you_re_looking_for()

   // new
   let fuu = hypnotoad().all_glory_to_the_hypnotoad()
   ```
