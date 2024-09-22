# iceoryx2 v0.3.0

## [v0.3.0](https://github.com/eclipse-iceoryx/iceoryx2/tree/v0.3.0) (2024-18-04) <!--NOLINT remove this when tag is set-->

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v0.2.2...v0.3.0)

<!--NOLINT remove this when tag is set-->

### Features

* Add docker example
  [#83](https://github.com/eclipse-iceoryx/iceoryx2/issues/83)
* Introduce `iceoryx2-bb-posix::process_state` for process monitoring
  [#96](https://github.com/eclipse-iceoryx/iceoryx2/issues/96)
* Introduce concept `iceoryx2-cal::monitoring`
  [#96](https://github.com/eclipse-iceoryx/iceoryx2/issues/96)
* New constructs from
  [#123](https://github.com/eclipse-iceoryx/iceoryx2/issues/123)
    * Introduce semantic string `iceoryx2-bb-system-types::base64url`
    * Introduce `iceoryx2-cal::hash::HashValue` that contains the result of a hash
* Port `UsedChunkList` from iceoryx1
  [#129](https://github.com/eclipse-iceoryx/iceoryx2/issues/129)
* From [#133](https://github.com/eclipse-iceoryx/iceoryx2/issues/133)
    * Add `Notifier|Listener|Publisher|Subscriber::id()` method to acquire unique
    port id
    * Add `Sample::origin()` to determine the `UniquePublisherId` of the sender
* Performance improvements, especially for AMD CPUs
  [#136](https://github.com/eclipse-iceoryx/iceoryx2/issues/136)
* Introduce lock-free mpmc BitSet
  [#139](https://github.com/eclipse-iceoryx/iceoryx2/issues/139)
* Refactor Event API
  [#175](https://github.com/eclipse-iceoryx/iceoryx2/issues/175)
    * Add `event_id_max_value()` setting to `Event` service builder
    * Add `defaults.event.event_id_max_value` to config file (`iceoryx2.toml`)
    * Add `Listener::{try|timed|blocking}_wait_all` to grab a batch of `EventIds`
    to avoid infinite busy loop
* Example for complex data types
  [#175](https://github.com/eclipse-iceoryx/iceoryx2/issues/175)

### Bugfixes

* Fix undefined behavior in `spsc::{queue|index_queue}`
  [#87](https://github.com/eclipse-iceoryx/iceoryx2/issues/87)
* Fix `open_or_create` race
  [#108](https://github.com/eclipse-iceoryx/iceoryx2/issues/108)
* Fixes for [#116](https://github.com/eclipse-iceoryx/iceoryx2/issues/116)
    * Fix retrieve channel overflow caused by big publisher loans
    * Fix `CreationMode::OpenOrCreate` in `iceoryx2-bb-posix::SharedMemory`
    * Add missing memory synchronization to POSIX shm zero copy connection
    * Remove retrieve buffer full check from zero copy connection - sender had
    insufficient infos available
    * Fix data race in `iceoryx2-bb-lock-free::mpmc::Container`
* Fix insufficient memory reordering protection in `spsc::Queue::push` and
  `spsc::Queue::pop`
  [#119](https://github.com/eclipse-iceoryx/iceoryx2/issues/119)
* Fix data race due to operation reordering in `spmc::UnrestrictedAtomic::load`
  [#125](https://github.com/eclipse-iceoryx/iceoryx2/issues/125)
* Fix broken `Publisher|Subscriber::populate_{subscriber|publisher}_channels()`
  [#129](https://github.com/eclipse-iceoryx/iceoryx2/issues/129)
* Fix failing reacquire of delivered samples in the zero copy receive channel
  [#130](https://github.com/eclipse-iceoryx/iceoryx2/issues/130)
* Fix receiving of invalid samples when subscriber is connected
  [#131](https://github.com/eclipse-iceoryx/iceoryx2/issues/131)
* Fix problem where sample is released to the wrong publisher
  [#133](https://github.com/eclipse-iceoryx/iceoryx2/issues/133)
* Fix event notifier deadlock with reconnecting listeners
  [#139](https://github.com/eclipse-iceoryx/iceoryx2/issues/139)
* Fixes for FreeBSD 14.0
  [#140](https://github.com/eclipse-iceoryx/iceoryx2/issues/140)
    * Fix segfault in `iceoryx2-pal-posix;:shm_list()` caused by `sysctl`
    * Adjust test to handle unordered event notifications
* Fix non UTF-8 windows platform error messages
  [#145](https://github.com/eclipse-iceoryx/iceoryx2/issues/145)
* Correct inconsistent default config entries for windows
  [#149](https://github.com/eclipse-iceoryx/iceoryx2/issues/149)
* Fix that drop is not called when DynamicStorage is destroyed
  [#160](https://github.com/eclipse-iceoryx/iceoryx2/issues/160)
* Fix race in `UniqueSystemId` that leads to non-unique unique ids
  [#181](https://github.com/eclipse-iceoryx/iceoryx2/issues/181)

### Refactoring

* Replace `iceoryx2::service::Service` with `iceoryx2::service::Details`
  [#100](https://github.com/eclipse-iceoryx/iceoryx2/issues/100)
* Remove `'config` lifetime from all structs
  [#100](https://github.com/eclipse-iceoryx/iceoryx2/issues/100)
* Remove `UniqueIndex` returning method from
  `iceoryx2-bb-lock-free::mpmc::Container`, cannot be implemented correctly in
  our context [#116](https://github.com/eclipse-iceoryx/iceoryx2/issues/116)
* All `iceoryx2-cal::shared_memory` implementations use a `DynamicStorage`
  concept as base [#153](https://github.com/eclipse-iceoryx/iceoryx2/issues/153)
* Hardening DynamicStorage, storages with distinct types cannot be opened
  [#160](https://github.com/eclipse-iceoryx/iceoryx2/issues/160)
* IpcCapable handles explicity destroy underlying object in drop, instead of ref
  counting [#162](https://github.com/eclipse-iceoryx/iceoryx2/issues/162)

### New API features

* Add `FixedSizeByteString::from_bytes_truncated`
  [#56](https://github.com/eclipse-iceoryx/iceoryx2/issues/56)
* Add `Deref`, `DerefMut`, `Clone`, `Eq`, `PartialEq` and `extend_from_slice` to
  (FixedSize)Vec [#58](https://github.com/eclipse-iceoryx/iceoryx2/issues/58)
* `MessagingPattern` implements `Display`
  [#64](https://github.com/eclipse-iceoryx/iceoryx2/issues/64)
* Introduce traits for all ports (`Listen`, `Notify`, `Publish`, `DefaultLoan`,
  `UninitLoan`, `Subscribe`) and for samples (`PayloadMut`, `Payload`)
  [#69](https://github.com/eclipse-iceoryx/iceoryx2/issues/69)
* Implement `Ord` and `PartialOrd` for `FixedSizeByteString` and `ServiceName`
  [#110](https://github.com/eclipse-iceoryx/iceoryx2/issues/110)
* Remove `publish_subscribe::Header::time_stamp()` due to ordering and
  performance problems
  [#136](https://github.com/eclipse-iceoryx/iceoryx2/issues/136)

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

2. All port `Publisher`, `Subscriber`, `Listener` and `Notifier` no longer have
   a generic `'config` lifetime parameter.

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

3. `iceoryx2::service::Details` no longer has a generic `'config` lifetime
   parameter. `iceoryx2::service::Details` replaced
   `iceoryx2::service::Service`. All custom services need to implement
   `iceoryx2::service::Service`.

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

4. Writing functions with generic service parameter no longer require
   `Service + Details<'config>`. Now it suffices to just use `Service`

   ```rust
   // old
   fn my_generic_service_function<'config, ServiceType: iceoryx2::service::Service + iceoryx2::service::Details<'config>>();

   // new
   fn my_generic_service_function<ServiceType: iceoryx2::service::Service>();
   ```

5. Do not use `Header::time_stamp()`, when required make it part of the payload
   type. **Be aware, this can be expensive and can lead to a significantly
   increased latency!**

   ```rust
   // old
   let subscriber = service.subscriber().create()?;
   println!("sample timestamp: {:?}", sample.unwrap().header().time_stamp());

   // new
   use iceoryx2_bb_posix::clock::{Time, TimeBuilder};

   #[derive(Debug)]
   #[repr(C)]
   pub struct TimeStamp {
       seconds: u64,
       nanoseconds: u32,
   }

   impl TimeStamp {
       pub fn new() -> Self {
           let now = Time::now().unwrap();
           Self {
               seconds: now.seconds(),
               nanoseconds: now.nanoseconds(),
           }
       }
   }

   pub struct MyMessageType {
       payload: u64,
       time_stamp: TimeStamp
   }

   // sender side
   let publisher = service.publisher().create()?;
   let sample = publisher.loan_uninit()?;
   let sample = sample.write_payload(MyMessageType {
       payload: 1234,
       time_stamp: TimeStamp::now();
   });
   sample.send()?;

   // receiver side
   let subscriber = service.subscriber().create()?;
   println!("sample timestamp: {:?}", sample.unwrap().time_stamp);
   ```

6. `Listener::{try|timed|blocking}_wait_one()` replace
   `Listener::{try|timed|blocking}_wait()` and return an `Option<EventId>`
   instead of a slice of `EventId`s.

   ```rust
   // old
   if let Ok(events) = listener.try_wait() {
       for event_id in events {
           println!("event was triggered with id: {:?}", event_id);
       }
   }

   // new, iterative approach
   while let Ok(Some(event_id)) = listener.try_wait_one() {
       println!("event was triggered with id: {:?}", event_id);
   }

   // new, functional approach to grap a batch of id's
   listener.try_wait_all(|id| {
       println!("event was triggered with id: {:?}", event_id);
   });
   ```

7. Renamed method `EventId::as_u64()` to `EventId::as_value()`.

   ```rust
   // old
   println!("event id value {}", my_event_id.as_u64());

   // new
   println!("event id value {}", my_event_id.as_value());
   ```

8. `EventId`s have a max default value of `32`.

   ```rust
   // old
   let name = ServiceName::new("EventName")?;
   let event = zero_copy::Service::new(&name)
                   .event()
                   .open_or_create()?;
   let notifier = event.notifier().create()?;
   // leads now to `NotifierNotifyError::EventIdOutOfBounds`
   notifier.notify_with_custom_event_id(EventId::new(1234));

   // new
   let name = ServiceName::new("EventName")?;
   let event = zero_copy::Service::new(&name)
                   .event()
                   // supports event id values up to 2048
                   // be aware, the greater the max value the slower the
                   // event mechanism is
                   .event_id_max_value(2048)
                   .open_or_create()?;
   let notifier = event.notifier().create()?;
   notifier.notify_with_custom_event_id(EventId::new(1234));
   ```
