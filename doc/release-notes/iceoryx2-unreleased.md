# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x) (xxxx-xx-xx) <!--NOLINT remove this when tag is set-->

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x) <!--NOLINT remove this when tag is set-->

### Features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Bugfixes

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Refactoring

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Workflow

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### New API features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Add `FixedSizeByteString::from_bytes_truncated` [#56](https://github.com/eclipse-iceoryx/iceoryx2/issues/56)
 * Add `Deref`, `DerefMut`, `Clone`, `Eq`, `PartialEq` and `extend_from_slice` to (FixedSize)Vec [#58](https://github.com/eclipse-iceoryx/iceoryx2/issues/58)
 * `MessagingPattern` implements `Display` [#64](https://github.com/eclipse-iceoryx/iceoryx2/issues/64)
 * Introduce traits for all ports (`Listener`, `Notifier`, `Publisher`, `PublisherLoan`, `Subscriber`)
   and for samples (`SampleMut`, `Sample`) [#69](https://github.com/eclipse-iceoryx/iceoryx2/issues/69)

### API Breaking Changes

1. Use `SampleMut::send()` instead of `Publisher::send()`

    ```rust
    // old
    let publisher = service.publisher().create()?;
    let sample = publisher.loan()?;
    // set sample value
    publisher.send(sample)?;

    // new
    let publisher = service.publisher().create()?;
    let sample = publisher.loan()?;
    // set sample value
    sample.send()?;
    ```

2. Port types renamed, `Impl` suffix was added to all ports

    ```rust
    // old
    let publisher: Publisher<'_, '_, zero_copy::Service, u64> = service.publisher().create()?;

    // new
    let publisher: PublisherImpl<'_, '_, zero_copy::Service, u64> = service.publisher().create()?;

    // same applies also to:
    // * `Subscriber` -> `SubscriberImpl`
    // * `Listener` -> `ListenerImpl`
    // * `Notifier` -> `NotifierImpl`
    ```

