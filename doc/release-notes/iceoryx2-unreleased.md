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

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove trait re-exports from iceoryx2-bb-elementary
  [#757](https://github.com/eclipse-iceoryx/iceoryx2/issues/757)

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
