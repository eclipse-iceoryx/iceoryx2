# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x)

### Features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Subscriber buffer size can be reduced [#19](https://github.com/eclipse-iceoryx/iceoryx2/issues/19)

### Bugfixes

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Refactoring

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * `open`, `open_or_create` and `create` are untyped in pubsub-builder [#195](https://github.com/eclipse-iceoryx/iceoryx2/issues/195)

### Workflow

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### New API features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### API Breaking Changes

1. `open`, `open_or_create` and `create` are untyped for publish-subscribe services

    ```rust
    // old
    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe()
        .create::<u64>() // or open::<u64>(), or open_or_create::<u64>()
        .unwrap();

    // new
    let service = zero_copy::Service::new(&service_name)
        .publish_subscribe()
        .typed::<u64>()
        .create() // or open(), or open_or_create()
        .unwrap();
    ```
