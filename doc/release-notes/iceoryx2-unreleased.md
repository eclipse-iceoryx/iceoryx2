# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* All port factories implement `Send`
  [#768](https://github.com/eclipse-iceoryx/iceoryx2/issues/768)
* `iox2 service listen` and `iox2_service notify`
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

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove trait re-exports from iceoryx2-bb-elementary
  [#757](https://github.com/eclipse-iceoryx/iceoryx2/issues/757)
* Make POSIX user- and group details optional
  [#780](https://github.com/eclipse-iceoryx/iceoryx2/issues/780)

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

1. Example

   ```rust
   // old
   let fuu = hello().is_it_me_you_re_looking_for()

   // new
   let fuu = hypnotoad().all_glory_to_the_hypnotoad()
   ```
