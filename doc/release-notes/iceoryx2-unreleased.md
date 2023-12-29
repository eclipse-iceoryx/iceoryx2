# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x) (xxxx-xx-xx) <!--NOLINT remove this when tag is set-->

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x) <!--NOLINT remove this when tag is set-->

### Features

 * MacOS Platform support [#51](https://github.com/eclipse-iceoryx/iceoryx2/issues/51)
 * Services with the same name for different messaging patterns are supported [#16](https://github.com/eclipse-iceoryx/iceoryx2/issues/16)

### Bugfixes

 * Example [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Refactoring

 * Example [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Workflow

 * Example [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### New API features

 * Add `FixedSizeByteString::from_bytes_truncated` [#56](https://github.com/eclipse-iceoryx/iceoryx2/issues/56)

### API Breaking Changes

1. Example

    ```rust
    // old
    let fuu = hello().is_it_me_you_re_looking_for()

    // new
    let fuu = hypnotoad().all_glory_to_the_hypnotoad()
    ```
