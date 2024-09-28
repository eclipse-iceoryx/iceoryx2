# iceoryx2 v0.4.0

## [v0.4.0](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.4.0)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.3.0...v0.4.0)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Subscriber buffer size can be reduced
  [#19](https://github.com/eclipse-iceoryx/iceoryx2/issues/19)
* Nodes cleanup all resources of dead nodes on creation and destruction
  [#96](https://github.com/eclipse-iceoryx/iceoryx2/issues/96)
* CLI for iox2 [#98](https://github.com/eclipse-iceoryx/iceoryx2/issues/98)
    * Add `iox2` CLI
    * Add `iox2-service` CLI
    * Add `iox2-node` CLI
* Introduce Nodes [#102](https://github.com/eclipse-iceoryx/iceoryx2/issues/102)
    * Implement Serialize,Deserialize for
        * `SemanticString`
        * `UniqueSystemId`
* Nodes register in service to enable monitoring
  [#103](https://github.com/eclipse-iceoryx/iceoryx2/issues/103)
* Multiple features from
  [#195](https://github.com/eclipse-iceoryx/iceoryx2/issues/195)

    * Introduce `payload_alignment` in `publish_subscribe` builder to increase
    alignment of payload for all service samples
    * Introduce support for slice-types with dynamic sizes.
    * Introduce `max_slice_len` in the publisher builder to set support dynamic
    sized types (slices).

  ```rust
  use iceoryx2::prelude::*;

  fn main() -> Result<(), Box<dyn std::error::Error>> {
      let node = NodeBuilder::new().create::<ipc::Service>()?;
      let service = node.service_builder("My/Funk/ServiceName".try_into()?)
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
    * Introduce `IoxAtomic` that supports up to 128bit atomics on 32-bit
    architecture with a ReadWriteLock
    * add CI targets to officially support 32-bit
* Example that demonstrates publish-subscribe communication with dynamic data
  [#205](https://github.com/eclipse-iceoryx/iceoryx2/issues/205)
* Introduce `PlacementNew` trait and derive proc-macro
  [#224](https://github.com/eclipse-iceoryx/iceoryx2/issues/224)
* Custom service properties support, see
  [example](https://github.com/eclipse-iceoryx/iceoryx2/tree/main/examples/rust/service_properties)
  [#231](https://github.com/eclipse-iceoryx/iceoryx2/issues/231)
    * Implement Serialize,Deserialize for
        * `FixedSizeByteString`
        * `FixedSizeVec`
* TryInto implemented for `{Node|Service}Name`
  [#243](https://github.com/eclipse-iceoryx/iceoryx2/issues/243)
* Add custom user header
  [#253](https://github.com/eclipse-iceoryx/iceoryx2/issues/253)
* Build the C and C++ language bindings with bazel
  [#329](https://github.com/eclipse-iceoryx/iceoryx2/issues/329)
* Add `Subscriber::has_samples`
  [#335](https://github.com/eclipse-iceoryx/iceoryx2/issues/335)
* Example that demonstrates iceoryx2 domains
  [#370](https://github.com/eclipse-iceoryx/iceoryx2/issues/370)
* Add colcon package for iceoryx2 with C/C++ bindings
  [#381](https://github.com/eclipse-iceoryx/iceoryx2/issues/381)
* Lock-free atomics on 32-bit architectures
  [#401](https://github.com/eclipse-iceoryx/iceoryx2/issues/401)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Build failure for Windows 11 i686-pc-windows-msvc
  [#235](https://github.com/eclipse-iceoryx/iceoryx2/issues/235)
* 'win32call' needs to provide the last error
  [#241](https://github.com/eclipse-iceoryx/iceoryx2/issues/241)
* Mem-leak in `iceoryx2-bb-posix::Directory::contents()` and skip empty file
  names [#287](https://github.com/eclipse-iceoryx/iceoryx2/issues/287)
* Log macros do no longer return values
  [#292](https://github.com/eclipse-iceoryx/iceoryx2/issues/292)
* Fix cross-compilation issue with `scandir.c`
  [#318](https://github.com/eclipse-iceoryx/iceoryx2/issues/318)
* Fix sample loss when publisher disconnected before subscriber called receive
  [#337](https://github.com/eclipse-iceoryx/iceoryx2/issues/337)
* Service-creation-timeout is considered also for the data segment and zero copy
  connection [#361](https://github.com/eclipse-iceoryx/iceoryx2/issues/361)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Kebab case for config file
  [#90](https://github.com/eclipse-iceoryx/iceoryx2/issues/90)
* `open`, `open_or_create` and `create` are untyped in pubsub-builder
  [#195](https://github.com/eclipse-iceoryx/iceoryx2/issues/195)
* use `ClockMode::Performance` instead of `ClockMode::Safety` in default
  deployment [#207](https://github.com/eclipse-iceoryx/iceoryx2/issues/207)
* Updated all dependencies and increased MSRV to 1.75
  [#221](https://github.com/eclipse-iceoryx/iceoryx2/issues/221)
* Remove `Service::does_exist_with_custom_config` and
  `Service::list_with_custom_config`
  [#238](https://github.com/eclipse-iceoryx/iceoryx2/issues/238)
* Renamed `PortFactory::{publisher|subscriber|listener|notifier}` to
  `PortFactory::{publisher|subscriber|listener|notifier}_builder`
  [#244](https://github.com/eclipse-iceoryx/iceoryx2/issues/244)
* Merged `Iox2::wait` with new `Node` and removed `Iox2`
  [#270](https://github.com/eclipse-iceoryx/iceoryx2/issues/270)
* Renamed `zero_copy::Service` and `process_local::Service` into `ipc::Service`
  and `local::Service`
  [#323](https://github.com/eclipse-iceoryx/iceoryx2/issues/323)
* Introduce `SampleMutUninit<Payload>` without `send` functionality
  as replacement for `SampleMut<MaybeUninit<Payload>>`
  [#394](https://github.com/eclipse-iceoryx/iceoryx2/issues/394)

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Extended CLI parameters for benchmarks
  [#360](https://github.com/eclipse-iceoryx/iceoryx2/issues/360)
* Default log-level is set from `LogLevel::Trace` to `LogLevel::Info`
  [#392](https://github.com/eclipse-iceoryx/iceoryx2/issues/392)

### API Breaking Changes

1. Services are created via the `Node`, `service_builder` take `ServiceName` by
   value

   ```rust
   // old
   let service = zero_copy::Service::new(&service_name)
       .event()
       .create()?;

   // new
   let node = NodeBuilder::new().create::<ipc::Service>()?;
   let service = node.service_builder(service_name) // service_name is moved into builder
       .event()
       .create()?;
   ```

2. Custom configurations are added to the `Node`. Methods
   `{event|publish_subscribe}_with_custom_config` are removed

   ```rust
   // old
   let service = zero_copy::Service::new(&service_name)
       .publish_subscribe_with_custom_config::<u64>(&custom_config)
       .open_or_create()?;

   // new
   let node = NodeBuilder::new()
                   .config(&custom_config)
                   .create::<ipc::Service>()?;

   let service = node.service_builder(service_name)
       .publish_subscribe::<u64>()
       .open_or_create()?;
   ```

3. `open`, `open_or_create` and `create` are untyped for publish-subscribe
   services

   ```rust
   // old
   let service = zero_copy::Service::new(&service_name)
       .publish_subscribe()
       .create::<u64>()?; // or open::<u64>(), or open_or_create::<u64>()

   // new
   let node = NodeBuilder::new().create::<ipc::Service>()?;
   let service = node.service_builder(service_name)
       .publish_subscribe::<u64>() // type is now up here
       .create()?; // or open(), or open_or_create()
   ```

4. `service_name` was renamed into `name` for consistency reasons.

   ```rust
   let services = ipc::Service::list()?;

   for service in services {
       // old
       let name = service.service_name();

       // new
       let name = service.name();
   }
   ```

5. Directory entries in `Config` converted to type `Path`

   ```rust
   let mut config = Config::default();

   // old
   config.global.service.directory = "Some/Directory".into();

   // new
   config.global.service.directory = "Some/Directory".try_into()?;
   ```

6. Suffix/prefix entries in `Config` converted to type `FileName`

   ```rust
   let mut config = Config::default();

   // old
   config.global.prefix = "iox2_".into();

   // new
   config.global.prefix = "iox2_".try_into()?;

   ```

7. `Service::list_with_custom_config` was removed.

   ```rust
   // old
   let services = zero_copy::Service::list()?;
   let services = zero_copy::Service::list_with_custom_config(Config::global_config())?;

   // new
   let services = ipc::Service::list(Config::global_config())?;
   ```

8. `Service::does_exist_with_custom_config` was removed.

   ```rust
   // old
   let services = zero_copy::Service::does_exist(service_name)?;
   let services = zero_copy::Service::does_exist_with_custom_config(service_name, Config::global_config())?;

   // new
   let services = ipc::Service::does_exist(service_name, Config::global_config())?;
   ```

9. Creating pub-sub ports with `service.{publisher|subscriber}_builder()`.

   ```rust
   // old
   let publisher = service.publisher().create()?;
   let subscriber = service.subscriber().create()?;

   // new
   let publisher = service.publisher_builder().create()?;
   let subscriber = service.subscriber_builder().create()?;
   ```

10. Creating event ports with `service.{listener|notifier}_builder()`.

    ```rust
    // old
    let listener = service.listener().create()?;
    let notifier = service.notifier().create()?;

    // new
    let listener = service.listener_builder().create()?;
    let notifier = service.notifier_builder().create()?;
    ```

11. The keys in the `iceoryx2.toml` config file are now kebab-case

    ```toml
    # old
    [defaults.publish_subscribe]
    max_subscribers                             = 8
    max_publishers                              = 2

    # new
    [defaults.publish-subscribe]
    max-subscribers                             = 8
    max-publishers                              = 2
    ```

12. Some error enum fields where removed or renamed.

    ```test
    // old                                                                  new
    EventOpenError::EventInCorruptedState                                   EventOpenError::ServiceInCorruptedState
    EventOpenError::PermissionDenied                                        EventOpenError::InsufficientPermissions
    EventOpenError::UnableToOpenDynamicServiceInformation                   EventOpenError::ServiceInCorruptedState

    EventCreateError::PermissionDenied                                      EventCreateError::InsufficientPermissions
    EventCreateError::UnableToCreateStaticServiceInformation                EventCreateError::ServiceInCorruptedState

    PublishSubscribeOpenError::PermissionDenied                             PublishSubscribeOpenError::InsufficientPermissions
    PublishSubscribeOpenError::Inaccessible                                 PublishSubscribeOpenError::InsufficientPermissions
    PublishSubscribeOpenError::UnableToOpenDynamicServiceInformation        PublishSubscribeOpenError::ServiceInCorruptedState

    PublishSubscribeCreateError::PermissionDenied                           PublishSubscribeCreateError::InsufficientPermissions
    PublishSubscribeCreateError::Corrupted                                  PublishSubscribeCreateError::ServiceInCorruptedState
    PublishSubscribeCreateError::UnableToCreateStaticServiceInformation     PublishSubscribeCreateError::ServiceInCorruptedState

    PublisherLoanError::ExceedsMaxLoanedChunks                              PublisherLoanError::ExceedsMaxLoanedSamples
    ```

13. Switch order of `Service` and `Payload` parameter in `Sample` and
    `SampleMut` to be consistent with all other constructs. Add third parameter
    user header.

    ```rust
    // old
    struct SomeSamples {
        sample_mut: SampleMut<MyMessageType, zero_copy::Service>,
        sample: Sample<MyMessageType, zero_copy::Service>,
    }

    // new
    struct SomeSamples {
        sample_mut: SampleMut<ipc::Service, MyMessageType, ()>,
        sample: Sample<ipc::Service, MyMessageType, ()>,
    }
    ```

14. The `Sample`, `SampleMut`, `Publisher`, `Subscriber` and the
    `publish_subscribe::PortFactory` (the service ) have additional generic
    argument that represents the user header type. By default the user header is
    `()`.

    ```rust
    // old
    struct SomeStruct {
        service: publish_subscribe::PortFactory<zero_copy::Service, MyMessageType>,
        subscriber: Subscriber<zero_copy::Service, MyMessageType>,
        publisher: Publisher<zero_copy::Service, MyMessageType>,
        list_of_mut_samples: Vec<SampleMut<MyMessageType, zero_copy::Service>>,
        list_of_samples: Vec<Sample<MyMessageType, zero_copy::Service>>,
    }

    // new, no custom user header
    struct SomeStruct {
        service: publish_subscribe::PortFactory<ipc::Service, MyMessageType, ()>,
        subscriber: Subscriber<ipc::Service, MyMessageType, ()>,
        publisher: Publisher<ipc::Service, MyMessageType, ()>,
        list_of_mut_samples: Vec<SampleMut<ipc::Service, MyMessageType, ()>>,
        list_of_samples: Vec<Sample<ipc::Service, MyMessageType, ()>>,
    }

    // new, with custom user header
    struct SomeStruct {
        service: publish_subscribe::PortFactory<ipc::Service, MyMessageType, MyCustomHeader>,
        subscriber: Subscriber<ipc::Service, MyMessageType, MyCustomHeader>,
        publisher: Publisher<ipc::Service, MyMessageType, MyCustomHeader>,
        list_of_mut_samples: Vec<SampleMut<ipc::Service, MyMessageType, MyCustomHeader>>,
        list_of_samples: Vec<Sample<ipc::Service, MyMessageType, MyCustomHeader>>,
    }
    ```

15. To avoid heap allocations, `Service::list()` requires a callback that is
    called for every service entry instead of returning a `Vec`.

    ```rust
    // old
    let services = zero_copy::Service::list(Config::global_config())?;

    for service in services {
        println!("\n{:#?}", &service);
    }

    // new
    ipc::Service::list(Config::global_config(), |service| {
        println!("\n{:#?}", &service?);
        Ok(CallbackProgression::Continue)
    })?;
    ```

16. Rename `max_supported_{publisher,subscriber,notifier,listener}` into
    `max_{publisher,subscriber,notifier,lister}` in the services `PortFactory`.

    ```rust
    let event_service = node.service_builder("MyEventName".try_into()?)
                     .event()
                     .open_or_create()?;

    let pubsub_service = node.service_builder("MyPubSubName".try_into()?)
                     .publish_subscribe<u64>()
                     .open_or_create()?;

    // old
    event_service.static_config().max_supported_listeners();
    event_service.static_config().max_supported_notifier();

    pubsub_service.static_config().max_supported_publisher();
    pubsub_service.static_config().max_supported_subscriber();

    // new
    event_service.static_config().max_listeners();
    event_service.static_config().max_notifier();

    pubsub_service.static_config().max_publisher();
    pubsub_service.static_config().max_subscriber();
    ```

17. `Iox2::wait()` is part of the `Node`, `Iox2Event` renamed to `NodeEvent`

    ```rust
    // old
    while let Iox2Event::Tick = Iox2::wait(CYCLE_TIME) {
        // main loop stuff
    }

    // new
    let node = NodeBuilder::new().create::<ipc::Service>()?;
    while let NodeEvent::Tick = node.wait(CYCLE_TIME) {
        // main loop stuff
    }
    ```

18. Renamed `Config::get_global_config` to just `Config::global_config`

19. Renamed `process_local::Service` into `local::Service` and
    `zero_copy::Service` into `ipc::Service`.

    ```rust
    // old
    let node = NodeBuilder::new().create::<zero_copy::Service>()?;
    let node = NodeBuilder::new().create::<process_local::Service>()?;

    // new
    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let node = NodeBuilder::new().create::<local::Service>()?;
    ```

20. `SampleMutUninit` without `send` as replacement for `SampleMut<MaybeUninit<Payload>>`

    ```rust
    // old
    let sample: SampleMut<zero_copy::Service, MaybeUninit<u64>> = publisher.loan_uninit()?;
    let sample = sample.write_payload(123);
    let sample.send()?;

    // new
    let sample: SampleMutUninit<zero_copy::Service, MaybeUninit<u64>> = publisher.loan_uninit()?;
    let sample = sample.write_payload(123);
    let sample.send()?;

    // no longer compiles
    let sample = publisher.loan_uninit()?;
    // uninitialized samples cannot be sent
    let sample.send()?;
    ```

## Thanks To All Contributors Of This Version

* [Christian »elfenpiff« Eltzschig](https://github.com/elfenpiff)
* [Mathias »elBoberido« Kraus](https://github.com/elboberido)
* [»orecham«](https://github.com/orecham)
