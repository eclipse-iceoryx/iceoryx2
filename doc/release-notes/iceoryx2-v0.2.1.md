# iceoryx2 v0.2.1

## [v0.2.1](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.2.1)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.1.1...v0.2.1)

### Features

* macOS Platform support
  [#51](https://github.com/eclipse-iceoryx/iceoryx2/issues/51)
* Services with the same name for different messaging patterns are supported
  [#16](https://github.com/eclipse-iceoryx/iceoryx2/issues/16)

### Bugfixes

* Fix undefined behavior in `FixedSizeByteString::new_unchecked`
  [#61](https://github.com/eclipse-iceoryx/iceoryx2/issues/61)
* Fix suffix of static config
  [#66](https://github.com/eclipse-iceoryx/iceoryx2/issues/66)
* Interpret non-existing service directory as no existing services
  [#63](https://github.com/eclipse-iceoryx/iceoryx2/issues/63)

### Refactoring

* Rename char in platform to C_char
  [#54](https://github.com/eclipse-iceoryx/iceoryx2/issues/54)
* Set reasonable Rust min version to 1.70 and verify it with additional CI
  targets [#72](https://github.com/eclipse-iceoryx/iceoryx2/issues/72)

### Workflow

* add `cargo audit` for security vulnerability checking in dependencies
  [#48](https://github.com/eclipse-iceoryx/iceoryx2/issues/48)

### New API features

* Add `FixedSizeByteString::from_bytes_truncated`
  [#56](https://github.com/eclipse-iceoryx/iceoryx2/issues/56)
* Add `Deref`, `DerefMut`, `Clone`, `Eq`, `PartialEq` and `extend_from_slice` to
  (FixedSize)Vec [#58](https://github.com/eclipse-iceoryx/iceoryx2/issues/58)
* `MessagingPattern` implements `Display`
  [#64](https://github.com/eclipse-iceoryx/iceoryx2/issues/64)

## Thanks To All Contributors Of This Version

* [Christian »elfenpiff« Eltzschig](https://github.com/elfenpiff)
* [Mathias »elBoberido« Kraus](https://github.com/elboberido)
* [»Shock-1«](https://github.com/Shock-1)
* [»hydroid7«](https://github.com/hydroid7)
