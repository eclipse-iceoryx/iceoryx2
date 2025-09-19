# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Remove duplicate entries in `iox2` command search path to prevent discovered
  commands from being listed multiple times
    [#1045](https://github.com/eclipse-iceoryx/iceoryx2/issues/1045)
* Print help for positional arguments in CLI
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)
* Print new line after CLI output to prevent '%' from being inserted by terminal
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Example text [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1)

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add end-to-end tests for `iceoryx2-cli`
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* Add option to force overwrite configuration with `iox2 config generate`
    [#709](https://github.com/eclipse-iceoryx/iceoryx2/issues/709)

### API Breaking Changes

1. Example

   ```rust
   // old
   let fuu = hello().is_it_me_you_re_looking_for()

   // new
   let fuu = hypnotoad().all_glory_to_the_hypnotoad()
   ```

1. Add summarized and detailed variants of `iox2 service discovery`

   ```console
   // old
   $ iox2 service discovery
   === Service Started (rate: 100ms) ===
   Added((
       service_id: ("4eacadf2695a3f4b2eb95485759246ce1a2aa906"),
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       messaging_pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
   ))
   Removed((
       service_id: ("4eacadf2695a3f4b2eb95485759246ce1a2aa906"),
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       messaging_pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
   ))

   // new
   $ iox2 service discovery
   Discovering Services (rate: 100ms)
   Added(PublishSubscribe("My/Funk/ServiceName"))
   Removed(PublishSubscribe("My/Funk/ServiceName"))

   $ iox2 service discovery --detailed
   Discovering Services (rate: 100ms)
   Added((
       service_id: "4eacadf2695a3f4b2eb95485759246ce1a2aa906",
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
       nodes: Some((
           num: 1,
           details: [
               (
                   state: Alive,
                   id: ("0000000034fcd3b8000013a8000135c1"),
                   pid: 79297,
                   executable: Some("publish_subscribe_subscriber"),
                   name: Some(""),
               ),
           ],
       )),
   ))
   Removed((
       service_id: "4eacadf2695a3f4b2eb95485759246ce1a2aa906",
       service_name: "My/Funk/ServiceName",
       attributes: ([]),
       pattern: PublishSubscribe((
           max_subscribers: 8,
           max_publishers: 2,
           max_nodes: 20,
           history_size: 0,
           subscriber_max_buffer_size: 2,
           subscriber_max_borrowed_samples: 2,
           enable_safe_overflow: true,
           message_type_details: (
               header: (
                   variant: FixedSize,
                   type_name: "iceoryx2::service::header::publish_subscribe::Header",
                   size: 40,
                   alignment: 8,
               ),
               user_header: (
                   variant: FixedSize,
                   type_name: "()",
                   size: 0,
                   alignment: 1,
               ),
               payload: (
                   variant: FixedSize,
                   type_name: "TransmissionData",
                   size: 16,
                   alignment: 8,
               ),
           ),
       )),
       nodes: Some((
           num: 1,
           details: [
               (
                   state: Alive,
                   id: ("0000000034fcd3b8000013a8000135c1"),
                   pid: 79297,
                   executable: Some("publish_subscribe_subscriber"),
                   name: Some(""),
               ),
           ],
       )),
   ))
   ```
