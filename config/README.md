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
* `global.service.creation-timeout.secs` - [int]: Maximum time for service setup
  in seconds. Uncreated services after this are marked as stalled.
  Attention: Both 'secs' and 'nanos' must be set together; leaving one
  unset will cause the configuration to be invalid.
* `global.service.creation-timeout.nanos` - [int]: Additional nanoseconds for
  service setup timeout.Maximum time for service setup.
  Attention: Both 'secs' and 'nanos' must be set together; leaving one
  unset will cause the configuration to be invalid.
* `global.service.blackboard-mgmt-suffix` - [string]: The suffix of the blackboard
management data segment
* `global.service.blackboard-data-suffix` - [string]: The suffix of the blackboard
payload data segment

## Defaults

### Service: Event Messaging Pattern

* `defaults.event.max-listeners` - [int]: Maximum number of listeners.
* `defaults.event.max-notifiers` - [int]: Maximum number of notifiers.
* `defaults.event.max-nodes` - [int]: Maximum number of nodes.
* `defaults.event.event-id-max-value` - [int]: Greatest value an [`EventId`] can
  have.
* `defaults.event.deadline` - [Option\<Duration\>]: Maximum allowed time between
  two consecutive notifications. If not sent after this time, all listeners attached
  to a WaitSet will be notified.
  Due to a current limitation, the keys are actually
  `defaults.event.deadline.secs` and `defaults.event.deadline.nanos`
  Attention: Both 'secs' and 'nanos' must be set together; leaving one
  unset will cause the configuration to be invalid.
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
* `defaults.publish-subscribe.subscriber-max-buffer-size` - [int]: Maximum buffer
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
* `defaults.request-response.max-borrowed-responses-per-pending-response` - [int]:
  The maximum number of borrowed responses a client can hold in
  parallel per pending response.
* `defaults.request-response.max-clients` - [int]:
  The maximum amount of supported clients.
* `defaults.request-response.max-nodes` - [int]:
  The maximum amount of supported nodes. Defines indirectly how many
  processes can open the service at the same time.
* `defaults.request-response.max-response-buffer-size` - [int]:
  The maximum buffer size for responses for an active request.
* `defaults.request-response.max-loaned-requests` - [int]:
  Maximum number of requests a client can loan in parallel.
* `defaults.request-response.server-max-loaned-responses-per-request` - [int]:
  Maximum number of responses a server can loan per request.
* `defaults.request-response.max-servers` - [int]:
  The maximum amount of supported servers.
* `defaults.request-response.server-unable-to-deliver-strategy` -
  [`Block`|`DiscardSample`]: Default strategy for non-overflowing setups
  when delivery fails.
* `defaults.request-response.server-expired-connection-buffer` - [int]:
  Expired connection buffer size of the server. Connections to clients
  are expired when the client disconnected from the service and the
  connection contains unconsumed active requests.

### Blackboard Pattern

* `defaults.blackboard.max-readers` - [int]: The maximum amount of supported Readers.
* `defaults.blackboard.max-nodes` - [int]: The maximum amount of supported Nodes.
Defines indirectly how many processes can open the service at the same time.

## Custom Platform Configuration

> [!WARNING]
> This feature is not available for Bazel builds.

The platform configuration in `iceoryx2-pal/configuration/src/lib.rs`
contains a variety of attributes for configuring `iceoryx2` for a
flexible deployment.
One example is the `ICEORYX2_ROOT_PATH` that contains the operational files for
services and nodes. These settings are defined at compile-time and the baseline
for the runtime configuration in TOML format as described above.

The default settings are already tailored to the most common
operating systems but it can happen that users may face a specific limitation
(e.g. the `TEMP_DIRECTORY` is not writeable) on their specific platform.
To provide the flexibility to easily handle such cases, it is possible to
define a custom platform configuration to be used in the build.

The first step is to create a file with custom configurtion
at any location (e.g. `/my/funky/platform/platform_configuration.rs`).

An example configuration may look like this, however be sure to check the
existing configurations at `iceoryx2-pal/configuration/src/lib.rs` as the
available attributes may have changed:

```rust
pub mod settings {
    pub const GLOBAL_CONFIG_PATH: &[u8] = b"/etc";
    pub const USER_CONFIG_PATH: &[u8] = b".config";
    pub const TEMP_DIRECTORY: &[u8] = b"/my_tmp/";
    pub const TEST_DIRECTORY: &[u8] = b"/my_tmp/tests/";
    pub const SHARED_MEMORY_DIRECTORY: &[u8] = b"/dev/my_shm/";
    pub const PATH_SEPARATOR: u8 = b'/';
    pub const ROOT: &[u8] = b"/";
    pub const ICEORYX2_ROOT_PATH: &[u8] = b"/my_tmp/";
    pub const FILENAME_LENGTH: usize = 255;
    // it is actually 4096 but to be more compatible with windows and also safe some stack the number
    // is reduced to 255
    pub const PATH_LENGTH: usize = 255;
    #[cfg(not(target_os = "macos"))]
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 0.25;
    #[cfg(target_os = "macos")]
    pub const AT_LEAST_TIMING_VARIANCE: f32 = 1.0;
}
```

To configure the build to use the custom settings, the following must be done:

1. Set environment variable with absolute path to the configuration file:

    ```cli
    export IOX2_CUSTOM_PLATFORM_CONFIGURATION_PATH=/my/funky/platform/platform_configuration.rs
    ```

    To make it persistent, the variable can be set in a `./cargo/config.toml`
    file, either globally or locally within the project:

    ```toml
    [env]
    IOX2_CUSTOM_PLATFORM_CONFIGURATION_PATH = "/my/funky/platform/platform_configuration.rs"
    ```

1. Build `iceoryx2` like normal, the configuration will be automatically
   substituted in at build time and a warning message will be emitted to
   confirm that the build has successfully built with the custom configuration

   ```console
   warning: iceoryx2-pal-configuration@x.y.z: Building with custom configuration: /my/funky/platform/platform_configuration.rs
   ```

## Custom Platform Abstraction Layer

> [!WARNING]
> This feature is not available for Bazel builds.

Similar to the platform configuration, users may want to use a completely
different platform abstraction (from those in
`iceoryx2/iceoryx2-pal/posix/src`) for a specific operating system.

To achieve this, the environment variable `IOX2_CUSTOM_POSIX_PLATFORM_PATH` can be
utilized.

The first step is to prepare the customized platform at some location (which
can be outside of the `iceoryx2` workspace).
For this example the path `/my/funky/platform/posix` is used.
A good starting point is to make a copy of one of the existing abstractions,
then adapt it to the specific needs.

The build steps are as follows:

1. Set the environment variable with an absolute path to custom platform
    abstraction (without a trailing slash)

    ```cli
    export IOX2_CUSTOM_POSIX_PLATFORM_PATH=/my/funky/platform/posix
    ```

    To make it persistent, the variable can be set in a `./cargo/config.toml`
    file, either globally or locally within the project:

    ```toml
    [env]
    IOX2_CUSTOM_POSIX_PLATFORM_PATH = "/my/funky/platform/posix"
    ```

1. Build `iceoryx2` like normal, the platform will be automatically substituted
   in at build time and a warning message will be emitted to confirm that the
   build has successfully built with the custom platform

   ```console
   warning: iceoryx2-pal-posix@x.y.z: Building with custom POSIX abstraction at: /my/funky/platform/posix/
   ```
