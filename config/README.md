# iceoryx2 Global Configuration

For streamlined multi-instance operation and interference-free communication,
iceoryx2 introduces global configuration settings. It enables the concurrent
execution of multiple iceoryx2 setups on the same machine or within a single
process by employing distinct configurations.

When **iceoryx2** is started without an explicitly loaded configuration,
such as:

```rust
use iceoryx2::config::Config;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_container::semantic_string::SemanticString;

Config::setup_global_config_from_file(
    &FilePath::new(b"my/custom/config/file.toml")?)?;
```

it will automatically search for a configuration file in the following
locations, in order:

1. `$PWD/config/iceoryx2.toml`
2. `$HOME/.config/iceoryx2/iceoryx2.toml`
3. `/etc/iceoryx2/iceoryx2.toml`

If no configuration file is found in these locations, **iceoryx2** will use
its default settings.

## Note

* The command
  ```cli
  iox2 config generate local
  ```
  automatically generates the `config.toml` file at `$HOME/.config/iceoryx2/iceoryx2.toml`.
* The command
  ```cli
  iox2 config generate global
  ```
  automatically generates the `config.toml` file at `/etc/iceoryx2/iceoryx2.toml`.

## Sections

The configuration is organized into two main sections:

* `global`: Contains settings affecting the entire deployment.
* `defaults`: Specifies default settings for quality of services and behaviors.

Adjusting `global` settings ensures a non-interfering setup.

## Global

* `global.root-path` - [string]: Defines the path for all
  iceoryx2 files and directories.
* `global.prefix` - [string]: Prefix that is used for every file iceoryx2
  creates.

### Nodes

* `global.node.directory` - [string]: Specifies the path for node-related files
  under `global.root-path`.
* `global.node.monitor-suffix` - [string]: Suffix added to the node monitor.
* `global.node.static-config-suffix` - [string]: Suffix added to the static
  config of the node.
* `global.node.service-tag-suffix` - [string]: Suffix added to the service tag
  of the node.
* `global.node.cleanup-dead-nodes-on-creation` - [`true`|`false`]: Defines if
  there shall be a scan for dead nodes with a following stale resource cleanup
  whenever a new node is created.
* `global.node.cleanup-dead-nodes-on-destruction` - [`true`|`false`]: Defines if
  there shall be a scan for dead nodes with a following stale resource cleanup
  whenever a node is going out-of-scope.

### Services

* `global.service.directory` - [string]: Specifies the path for service-related
  files under `global.root-path`.
* `global.service.data-segment-suffix` - [string]: Suffix added to the
  ports's data segment.
* `global.service.static-config-storage-suffix` - [string]: Suffix for static
  service configuration files.
* `global.service.dynamic-config-storage-suffix` - [string]: Suffix for dynamic
  service configuration files.
* `global.service.event-connection-suffix` - [string]: Suffix for event channel.
* `global.service.connection-suffix` - [string]: Suffix for one-to-one
  connections.
* `global.service.creation-timeout.secs` &
  `global.service.creation-timeout.nanos` - [int]: Maximum time for service
  setup. Uncreated services after this are marked as stalled.

## Defaults

### Service: Event Messaging Pattern

* `defaults.event.max-listeners` - [int]: Maximum number of listeners.
* `defaults.event.max-notifiers` - [int]: Maximum number of notifiers.
* `defaults.event.max-nodes` - [int]: Maximum number of nodes.
* `defaults.event.event-id-max-value` - [int]: Greatest value an [`EventId`] can
  have.
* `defaults.event.notifier-created-event` - [Option\<int\>]: If defined,
    it defines the event id that is emitted when a new notifier is created.
* `defaults.event.notifier-dropped-event` - [Option\<int\>]: If defined,
    it defines the event id that is emitted when a notifier is destroyed.
* `defaults.event.notifier-dead-event` - [Option\<int\>]: If defined,
    it defines the event id that is emitted when a dead notifier is cleaned up.

### Service: Publish Subscribe Messaging Pattern

* `defaults.publish-subscribe.max-subscribers` - [int]: Maximum number of
  subscribers.
* `defaults.publish-subscribe.max-publishers` - [int]: Maximum number of
  publishers.
* `defaults.publish-subscribe.max-nodes` - [int]: Maximum number of nodes.
* `defaults.publish-subscribe.publisher-history-size` - [int]: Maximum history
  size a subscriber can request.
* `defaults.publish-subscribe.subscriber-buffer-size` - [int]: Maximum buffer
  size of a subscriber.
* `defaults.publish-subscribe.subscriber-max-borrowed-samples` - [int]: Maximum
  samples a subscriber can hold.
* `defaults.publish-subscribe.publisher-max-loaned-samples` - [int]: Maximum
  samples a publisher can loan.
* `defaults.publish-subscribe.enable-safe-overflow` - [`true`|`false`]: Default
  overflow behavior.
* `defaults.publish-subscribe.unable-to-deliver-strategy` -
  [`Block`|`DiscardSample`]: Default strategy for non-overflowing setups when
  delivery fails.
* `defaults.publish-subscribe.subscriber-expired-connection-buffer` - [int]:
  Expired connection buffer size of the subscriber. Connections to publishers
  are expired when the publisher disconnected from the service and the
  connection contains unconsumed samples.

### Service: Request Response Messaging Pattern

* `defaults.request-response.client-unable-to-deliver-strategy` -
  [`Block`|`DiscardSample`]: Default strategy for non-overflowing setups
  when delivery fails.
* `defaults.request-response.client-expired-connection-buffer` - [int]:
  Expired connection buffer size of the client. Connections to servers
  are expired when the server disconnected from the service and the
  connection contains unconsumed responses.
* `defaults.request-response.enable-fire-and-forget-requests` -
  [`true`|`false`]: Enables the client to send requests without
  expecting a response.
* `defaults.request-response.enable-safe-overflow-for-requests` -
  [`true`|`false`]: Defines if the request buffer of the service safely
  overflows.
* `defaults.request-response.enable-safe-overflow-for-responses` -
  [`true`|`false`]: Defines if the request buffer of the service safely
  overflows.
* `defaults.request-response.max-active-requests-per-client` - [int]:
  The maximum of active requests a server can hold per client
* `defaults.request-response.max-borrowed-responses-per-pending-response` -
  [int]: The maximum number of borrowed responses a client can hold in
  parallel per pending response.
* `defaults.request-response.max-clients` - [int]:
  The maximum amount of supported clients.
* `defaults.request-response.max-nodes` - [int]:
  The maximum amount of supported nodes. Defines indirectly how many
  processes can open the service at the same time.
* `defaults.request-response.max-response-buffer-size` - [int]:
  The maximum buffer size for responses for an active request.
* `defaults.request-response.max-request-buffer-size` - [int]:
  The maximum buffer size for requests for a server.
* `defaults.request-response.max-servers` - [int]:
  The maximum amount of supported servers.
* `defaults.request-response.server-unable-to-deliver-strategy` -
  [`Block`|`DiscardSample`]: Default strategy for non-overflowing setups
  when delivery fails.
* `defaults.request-response.server-expired-connection-buffer` - [int]:
  Expired connection buffer size of the server. Connections to clients
  are expired when the client disconnected from the service and the
  connection contains unconsumed active requests.
