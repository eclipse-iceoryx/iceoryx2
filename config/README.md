# Iceoryx2 Global Configuration

For streamlined multi-instance operation and interference-free communication,
Iceoryx2 introduces global configuration settings. It enables the concurrent
execution of multiple Iceoryx2 setups on the same machine or within a single
process by employing distinct configurations.

## Sections

The configuration is organized into two main sections:

 * `global`: Contains settings affecting the entire deployment.
 * `defaults`: Specifies default settings for quality of services and behaviors.

Adjusting `global` settings ensures a non-interfering setup.

## Entries

### Global

 * `global.root_path` - [string]: Defines the path for all Iceoryx2 files and directories.
 * `global.prefix` - [string]: Prefix that is used for every file Iceoryx2 creates.
 * `global.service.directory` - [string]: Specifies the path for service-related files under `global.root_path`.
 * `global.service.publisher_data_segment_suffix` - [string]: Suffix added to the publisher's data segment.
 * `global.service.static_config_storage_suffix` - [string]: Suffix for static service configuration files.
 * `global.service.dynamic_config_storage_suffix` - [string]: Suffix for dynamic service configuration files.
 * `global.service.connection_suffix` - [string]: Suffix for one-to-one connections.
 * `global.service.creation_timeout.secs` & `global.service.creation_timeout.nanos` - [int]: Maximum time for service setup. Uncreated services after this are marked as stalled.

### Defaults

 * `defaults.publish_subscribe.max_subscribers` - [int]: Maximum number of subscribers.
 * `defaults.publish_subscribe.max_publishers` - [int]: Maximum number of publishers.
 * `defaults.publish_subscribe.publisher_history_size` - [int]: Maximum history size a subscriber can request.
 * `defaults.publish_subscribe.subscriber_buffer_size` - [int]: Maximum buffer size of a subscriber.
 * `defaults.publish_subscribe.subscriber_max_borrowed_samples` - [int]: Maximum samples a subscriber can hold.
 * `defaults.publish_subscribe.publisher_max_loaned_samples` - [int]: Maximum samples a publisher can loan.
 * `defaults.publish_subscribe.enable_safe_overflow` - [`true`|`false`]: Default overflow behavior.
 * `defaults.publish_subscribe.unable_to_deliver_strategy` - [`block`|`discard_sample`]: Default strategy for non-overflowing setups when delivery fails.
 * `defaults.event.max_listeners` - [int]: Maximum number of listeners.
 * `defaults.event.max_notifiers` - [int]: Maximum number of notifiers.
