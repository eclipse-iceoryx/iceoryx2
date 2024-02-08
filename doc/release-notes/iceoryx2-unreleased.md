# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x) (xxxx-xx-xx) <!--NOLINT remove this when tag is set-->

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x) <!--NOLINT remove this when tag is set-->

### Features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Introduce `iceoryx2-bb-posix::process_state` for process monitoring [#96](https://github.com/eclipse-iceoryx/iceoryx2/issues/96)
 * Introduce concept `iceoryx2-cal::monitoring` [#96](https://github.com/eclipse-iceoryx/iceoryx2/issues/96)

### Bugfixes

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Fix `open_or_create` race [#108](https://github.com/eclipse-iceoryx/iceoryx2/issues/108)
 * Fix undefined behavior in `spsc::{queue|index_queue}` [#87](https://github.com/eclipse-iceoryx/iceoryx2/issues/87)

### Refactoring

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Replace `iceoryx2::service::Service` with `iceoryx2::service::Details` [#100](https://github.com/eclipse-iceoryx/iceoryx2/issues/100)
 * Remove `'config` lifetime from all structs  [#100](https://github.com/eclipse-iceoryx/iceoryx2/issues/100)

### Workflow

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### New API features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Add `FixedSizeByteString::from_bytes_truncated` [#56](https://github.com/eclipse-iceoryx/iceoryx2/issues/56)
 * Add `Deref`, `DerefMut`, `Clone`, `Eq`, `PartialEq` and `extend_from_slice` to (FixedSize)Vec [#58](https://github.com/eclipse-iceoryx/iceoryx2/issues/58)
 * `MessagingPattern` implements `Display` [#64](https://github.com/eclipse-iceoryx/iceoryx2/issues/64)
 * Introduce traits for all ports (`Listen`, `Notify`, `Publish`, `DefaultLoan`, `UninitLoan`, `Subscribe`)
   and for samples (`PayloadMut`, `Payload`) [#69](https://github.com/eclipse-iceoryx/iceoryx2/issues/69)
 * Implement `Ord` and `PartialOrd` for `FixedSizeByteString` and `ServiceName` [#110](https://github.com/eclipse-iceoryx/iceoryx2/issues/110)

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

2. All port `Publisher`, `Subscriber`, `Listener` and `Notifier` no longer have a generic
    `'config` lifetime parameter.

    ```rust
    // old
    let publisher: Publisher<'service, 'config, iceoryx2::service::zero_copy::Service::Type<'config>, MessageType> = ..;
    let subscriber: Subscriber<'service, 'config, iceoryx2::service::zero_copy::Service::Type<'config>, MessageType> = ..;
    let notifier: Notifier<'service, 'config, iceoryx2::service::zero_copy::Service::Type<'config>> = ..;
    let listener: Listener<'service, 'config, iceoryx2::service::zero_copy::Service::Type<'config>> = ..;

    // new
    let publisher: Publisher<'service, iceoryx2::service::zero_copy::Service, MessageType> = ..;
    let subscriber: Subscriber<'service, iceoryx2::service::zero_copy::Service, MessageType> = ..;
    let notifier: Notifier<'service, iceoryx2::service::zero_copy::Service> = ..;
    let listener: Listener<'service, iceoryx2::service::zero_copy::Service> = ..;
    ```

3. `iceoryx2::service::Details` no longer has a generic `'config` lifetime parameter.
   `iceoryx2::service::Details` replaced `iceoryx2::service::Service`. All custom services need
   to implement `iceoryx2::service::Service`.

    ```rust
    // old
    pub struct MyCustomServiceType<'config> {
        state: ServiceState<'config, static_storage::whatever::Storage, dynamic_storage::whatever::Storage<WhateverConfig>>
    }

    impl<'config> crate::service::Service for MyCustomServiceType<'config> {
        // ...
    }

    impl<'config> crate::service::Details for MyCustomServiceType<'config> {
        // ...
    }

    // new
    pub struct MyCustomServiceType {
        state: ServiceState<static_storage::whatever::Storage, dynamic_storage::whatever::Storage<WhateverConfig>>
    }

    impl crate::service::Service for MyCustomServiceType {
        // ...
    }
    ```

4. Writing functions with generic service parameter no longer require `Service + Details<'config>`.
   Now it suffices to just use `Service`

    ```rust
    // old
    fn my_generic_service_function<'config, ServiceType: iceoryx2::service::Service + iceoryx2::service::Details<'config>>();

    // new
    fn my_generic_service_function<ServiceType: iceoryx2::service::Service>();
    ```
