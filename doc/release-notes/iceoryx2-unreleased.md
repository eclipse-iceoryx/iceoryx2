# iceoryx2 v?.?.?

## [vx.x.x](https://github.com/eclipse-iceoryx/iceoryx2/tree/vx.x.x)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/vx.x.x...vx.x.x)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Full C/C++ Language bindings for all features
    [#264](https://github.com/eclipse-iceoryx/iceoryx2/issues/264)
* Read LogLevel from environment variable
    [#396](https://github.com/eclipse-iceoryx/iceoryx2/issues/396)
* Add Request-Response messaging pattern
    [#429](https://github.com/eclipse-iceoryx/iceoryx2/issues/429)
* Lookup config file in default locations
    [#442](https://github.com/eclipse-iceoryx/iceoryx2/issues/442)
* Introduce `socket_pair` abstraction in POSIX wrapper
    [#508](https://github.com/eclipse-iceoryx/iceoryx2/issues/508)
* Introduce `socket_pair` event concept
    [#508](https://github.com/eclipse-iceoryx/iceoryx2/issues/508)
* Deadline property for event services
    [#573](https://github.com/eclipse-iceoryx/iceoryx2/issues/573)
* Use 'std_instead_of_core' clippy warning
    [#579](https://github.com/eclipse-iceoryx/iceoryx2/issues/579)
* Use 'std_instead_of_alloc' and 'alloc_instead_of_core' clippy warning
    [#581](https://github.com/eclipse-iceoryx/iceoryx2/issues/581)
* Introduce platform abstraction based on the 'libc' crate
    [#604](https://github.com/eclipse-iceoryx/iceoryx2/issues/604)
* Extend benchmarks to test setups with multiple sending/receiving ports
    [#610](https://github.com/eclipse-iceoryx/iceoryx2/issues/610)
* Reduce iceoryx2 dependencies
    [#640](https://github.com/eclipse-iceoryx/iceoryx2/issues/640)
* Allow customizable payload and user header type name in C++ binding
    [#643](https://github.com/eclipse-iceoryx/iceoryx2/issues/643)
* Expose set_log_level_from_env* APIs to C++
    [#653](https://github.com/eclipse-iceoryx/iceoryx2/issues/653)
* Introduce a "service discovery service", which applications can
    subscribe and listen to for updates in service landscape
    [#674](https://github.com/eclipse-iceoryx/iceoryx2/issues/674)
* Details of all service ports can be acquired
    [#685](https://github.com/eclipse-iceoryx/iceoryx2/issues/685)
* Add benchmark for request-response
    [#687](https://github.com/eclipse-iceoryx/iceoryx2/issues/687)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Corrupted services are removed when they are part of a dead node
    [#458](https://github.com/eclipse-iceoryx/iceoryx2/issues/458)
* Remove stale shm state files in Windows
    [#458](https://github.com/eclipse-iceoryx/iceoryx2/issues/458)
* Make process local services truly process local by using socket pairs
    for events
    [#508](https://github.com/eclipse-iceoryx/iceoryx2/issues/508)
* Completion queue capacity exceeded when history > buffer
    [#571](https://github.com/eclipse-iceoryx/iceoryx2/issues/571)
* Increase max supported shared memory size in Windows that restricts
    the maximum supported payload size
    [#575](https://github.com/eclipse-iceoryx/iceoryx2/issues/575)
* Undefined behavior due to ZeroCopyConnection removal when stale resources
    are cleaned up
    [#596](https://github.com/eclipse-iceoryx/iceoryx2/issues/596)
* Remove `SIGPOLL` that lead to compile issues on older glibc versions.
    Fix issue where fatal signals are generated with non-fatal values.
    [#605](https://github.com/eclipse-iceoryx/iceoryx2/issues/605)
* LogLevel is considered for custom loggers.
    [#608](https://github.com/eclipse-iceoryx/iceoryx2/issues/608)
* Allow missing legal characters in system type for user- and group-name
    [#677](https://github.com/eclipse-iceoryx/iceoryx2/issues/677)
* Fix `wait_and_process_once_with_timeout` deadlock
    [#695](https://github.com/eclipse-iceoryx/iceoryx2/issues/695)
* Fix Miri issues with MetaVec due to temporary borrow
    [#682](https://github.com/eclipse-iceoryx/iceoryx2/issues/682)
* Do not set default log level for cargo
    [#711](https://github.com/eclipse-iceoryx/iceoryx2/issues/711)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove the `print_system_configuration()` function in
`iceoryx2-bb/posix/src/system_configuration.rs` file and move it into the CLI `iox2-config`
    [#432](https://github.com/eclipse-iceoryx/iceoryx2/issues/432)
* Remove obsolete POSIX wrapper
    [#594](https://github.com/eclipse-iceoryx/iceoryx2/issues/594)
* Updated all dependencies and increased MSRV to 1.81
    [#638](https://github.com/eclipse-iceoryx/iceoryx2/issues/638)
* Reduce indentation in `main.rs` of CLI binaries
    [#646](https://github.com/eclipse-iceoryx/iceoryx2/issues/646)
* Improve naming in `AttributeSet` methods and `ServiceId`
    [#697](https://github.com/eclipse-iceoryx/iceoryx2/issues/697)
* Efficient `Clone` for `FixedSizeByteString`
    [#717](https://github.com/eclipse-iceoryx/iceoryx2/issues/717)

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add CLI to display complete system configuration
    [#432](https://github.com/eclipse-iceoryx/iceoryx2/issues/432)

* Add simplified attribute value accessors
    [#590](https://github.com/eclipse-iceoryx/iceoryx2/issues/590)

* Add CLI to launch service discovery service
    [#674](https://github.com/eclipse-iceoryx/iceoryx2/issues/674)

### API Breaking Changes

1. Add requirement that every payload and user header type must implement
   `ZeroCopySend` for type safe shared memory usage
   [#602](https://github.com/eclipse-iceoryx/iceoryx2/issues/602)

   ```rust
   // old
   #[repr(C)]
   pub struct TransmissionData {
      // ...
   }

   #[repr(C)]
   pub struct CustomHeader {
      // ...
   }

   let service = node
       .service_builder(&"ServiceName".try_into()?)
       .publish_subscribe::<TransmissionData>()
       .user_header::<CustomHeader>()
       .open_or_create()?;

   // new
   #[derive(ZeroCopySend)]
   #[repr(C)]
   pub struct TransmissionData {
      // ...
   }

   #[derive(ZeroCopySend)]
   #[repr(C)]
   pub struct CustomHeader {
      // ...
   }

   let service = node
       .service_builder(&"ServiceName".try_into()?)
       .publish_subscribe::<TransmissionData>()
       .user_header::<CustomHeader>()
       .open_or_create()?;
   ```

2. Renamed `PublisherLoanError` into `LoanError`

   ```rust
   // old
   let sample = match publisher.loan() {
     Ok(sample) => sample,
     Err(PublisherLoanError::OutOfMemory) => handle_error(),
     // ...
   };

   // new
   let sample = match publisher.loan() {
     Ok(sample) => sample,
     Err(LoanError::OutOfMemory) => handle_error(),
     // ...
   };
   ```

3. Renamed `PublisherSendError` into `SendError`

   ```rust
   // old
   match sample.send() {
     Ok(n) => println!("send data to {n} subscribers"),
     Err(PublisherSendError::ConnectionCorrupted) => handle_error(),
     // ...
   };

   // new
   match sample.send() {
     Ok(n) => println!("send data to {n} subscribers"),
     Err(SendError::ConnectionCorrupted) => handle_error(),
     // ...
   };
   ```

4. Renamed `SubscriberReceiveError` into `ReceiveError`

   ```rust
   // old
   match subscriber.receive() {
     Ok(sample) => println!("received: {:?}", *sample),
     Err(SubscriberReceiveError::ExceedsMaxBorrowedSamples) => handle_error(),
     // ...
   }

   // new
   match subscriber.receive() {
     Ok(sample) => println!("received: {:?}", *sample),
     Err(ReceiveError::ExceedsMaxBorrowedSamples) => handle_error(),
     // ...
   }
   ```

5. Renamed `PublisherSendError::ConnectionBrokenSincePublisherNoLongerExists`
   into `SendError::ConnectionBrokenSinceSenderNoLongerExists`

6. Renamed `ConnectionFailure::UnableToMapPublishersDataSegment`
   into `ConnectionFailure::UnableToMapSendersDataSegment`

7. Renamed `AttributeSet::len()`
   into `AttributeSet::number_of_attributes()`

8. Renamed `AttributeSet::get_key_value_len()`
   into `AttributeSet::number_of_key_values()`

9. Renamed `AttributeSet::get_key_value_at()`
   into `AttributeSet::key_value()`

10. Renamed `AttributeSet::get_key_values()`
   into `AttributeSet::iter_key_values()`

11. Renamed `ServiceId::max_len()`
   into `ServiceId::max_number_of_characters()`

12. The following types no longer implement `Copy`
   (the only implement `Clone`):

* `FixedSizeByteString`
* `SemanticString`
* `Base64URL`
* `FileName`
* `FilePath`
* `GroupName`
* `UserName`
* `ServiceId`
