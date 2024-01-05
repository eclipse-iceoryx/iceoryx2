# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x) (xxxx-xx-xx) <!--NOLINT remove this when tag is set-->

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x) <!--NOLINT remove this when tag is set-->

### Features

 * MacOS Platform support [#51](https://github.com/eclipse-iceoryx/iceoryx2/issues/51)
 * Services with the same name for different messaging patterns are supported [#16](https://github.com/eclipse-iceoryx/iceoryx2/issues/16)

### Bugfixes

 * Fix undefined behavior in `FixedSizeByteString::new_unchecked` [#61](https://github.com/eclipse-iceoryx/iceoryx2/issues/61)
 * Fix suffix of static config [#66](https://github.com/eclipse-iceoryx/iceoryx2/issues/66)
 * Interpret non-existing service directory as no existing services [#63](https://github.com/eclipse-iceoryx/iceoryx2/issues/63)

### Refactoring

 * Rename char in platform to c_char [#54](https://github.com/eclipse-iceoryx/iceoryx2/issues/54)
 * Set reasonable rust min version to 1.65 and verify it with additional CI targets [#72](https://github.com/eclipse-iceoryx/iceoryx2/issues/72)

### Workflow

 * add `cargo audit` for security vulnerability checking in dependencies [#48](https://github.com/eclipse-iceoryx/iceoryx2/issues/48)

### New API features

 * Add `FixedSizeByteString::from_bytes_truncated` [#56](https://github.com/eclipse-iceoryx/iceoryx2/issues/56)
 * Add `Deref`, `DerefMut`, `Clone`, `Eq`, `PartialEq` and `extend_from_slice` to (FixedSize)Vec [#58](https://github.com/eclipse-iceoryx/iceoryx2/issues/58)
 * `MessagingPattern` implements `Display` [#64](https://github.com/eclipse-iceoryx/iceoryx2/issues/64)

### API Breaking Changes

1. Example

    ```rust
    // old
    let fuu = hello().is_it_me_you_re_looking_for()

    // new
    let fuu = hypnotoad().all_glory_to_the_hypnotoad()
    ```
