# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x)

### Features

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Subscriber buffer size can be reduced [#19](https://github.com/eclipse-iceoryx/iceoryx2/issues/19)
 * Multiple features from [#195](https://github.com/eclipse-iceoryx/iceoryx2/issues/195)
    * Introduce `payload_alignment` in `publish_subscribe` builder to increase alignment of payload for all service samples
    * Introduce support for slice-types with dynamic sizes.
    * Introduce `max_slice_len` in the publisher builder to set support dynamic sized types (slices).

    ```rust
    use iceoryx2::prelude::*;

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let service_name = ServiceName::new("My/Funk/ServiceName")?;
        let service = zero_copy::Service::new(&service_name)
            .publish_subscribe::<[usize]>()
            // set a custom alignment of 512, interesting for SIMD-operations
            .payload_alignment(Alignment::new(512).unwrap())
            .open_or_create()?;

        let publisher = service
            .publisher()
            // defines the maximum length of a slice
            // can be set whenever a publisher is created to handle dynamic sized types
            .max_slice_len(128)
            .create()?;

        // loan some initialized memory and send it
        // the payload type must implement the [`core::default::Default`] trait in order to be able to use this API
        // we acquire a slice of length 12
        let mut sample = publisher.loan_slice(12)?;
        sample.payload_mut()[5] = 1337;
        sample.send()?;
    }
    ```
 * 32-bit support [#200](https://github.com/eclipse-iceoryx/iceoryx2/issues/200)
    * Introduce `IoxAtomic` that supports up to 128bit atomics on 32-bit architecture with a ReadWriteLock
    * add CI targets to officially support 32-bit
 * Example that demonstrates publish-subscribe communication with dynamic data [#205](https://github.com/eclipse-iceoryx/iceoryx2/issues/205)
 * Introduce `PlacementNew` trait and derive proc-macro [#224](https://github.com/eclipse-iceoryx/iceoryx2/issues/224)
 * Custom service properties support, see [example](https://github.com/eclipse-iceoryx/iceoryx2/tree/main/examples/rust/service_properties) [#231](https://github.com/eclipse-iceoryx/iceoryx2/issues/231)
    * Implement Serialize,Deserialize for
        * `FixedSizeByteString`
        * `FixedSizeVec`
 * Introduce Nodes [#102](https://github.com/eclipse-iceoryx/iceoryx2/issues/102)
    * Implement Serialize,Deserialize for
        * `SemanticString`
        * `UniqueSystemId`


### Bugfixes

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Refactoring

 <!-- NOTE: Add new entries sorted by issue number to minimize the possibility of conflicts when merging. -->

 * `open`, `open_or_create` and `create` are untyped in pubsub-builder [#195](https://github.com/eclipse-iceoryx/iceoryx2/issues/195)
 * use `ClockMode::Performance` instead of `ClockMode::Safety` in default deployment [#207](https://github.com/eclipse-iceoryx/iceoryx2/issues/207)
 * Updated all dependencies and increased MSRV to 1.75 [#221](https://github.com/eclipse-iceoryx/iceoryx2/issues/221)

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
        .publish_subscribe::<u64>() // type is now up here
        .create() // or open(), or open_or_create()
        .unwrap();
    ```

2. `service_name` was renamed into `name` for consistency reasons.

    ```rust
    let services = zero_copy::Service::list()?;

    for service in services {
        // old
        let name = service.service_name();

        // new
        let name = service.name();
    }
    ```

3. Directory entries in `Config` converted to type `Path`

    ```
    let mut config = Config::default();

    // old
    config.global.service.directory = "Some/Directory".into();

    // new
    config.global.service.directory = "Some/Directory".try_into()?;
    ```

4. Suffix/prefix entries in `Config` converted to type `FileName`

    ```
    let mut config = Config::default();

    // old
    config.global.prefix = "iox2_".into();

    // new
    config.global.prefix = "iox2_".try_into()?;
    ```
