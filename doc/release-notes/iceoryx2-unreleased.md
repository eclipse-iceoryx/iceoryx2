<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text
* [#2011](https://github.com/eclipse-iceoryx/iceoryx2/issues/2011) Add a publish-subscribe latency benchmark with a FlatBuffers payload

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text
* [#1951](https://github.com/eclipse-iceoryx/iceoryx2/issues/1951) Resolve the `.shm_state` directory at runtime and store it in a per-user directory (`IOX2_SHM_STATE_DIRECTORY` override, `%APPDATA%\iceoryx2\shm\` on Windows, `$XDG_STATE_HOME/iceoryx2/shm/` or `$HOME/.local/state/iceoryx2/shm/` on macOS/FreeBSD, `TEMP_DIRECTORY` fallback) instead of a shared system-wide temporary directory
* [#2018](https://github.com/eclipse-iceoryx/iceoryx2/issues/2018) Return `InvalidListenerKey` instead of panicking when notifying with a listener key whose index exceeds the service capacity

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1977](https://github.com/eclipse-iceoryx/iceoryx2/issues/1977) Restructure the ROS 2 gateway into a link gateway with a ROS 2 adapter
* [#1977](https://github.com/eclipse-iceoryx/iceoryx2/issues/1977) Remove the former gateway core in `iceoryx2-gateway/`, superseded by the link crates
* [#2010](https://github.com/eclipse-iceoryx/iceoryx2/issues/2010) Receive bytes from a tunnel directly into loaned samples

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1](https://github.com/eclipse-iceoryx/iceoryx2/issues/1) Example text

### API Breaking Changes

1. The gateway has been restructured to fit the link architecture, in which
   the link extends `iceoryx2` services across the boundary of a shared memory
   domain through an abstracted backend.

   The gateway is now one of the possible backends, in addition to the tunnel.
   Integration with the gateway is done via the adapter trait which specifies
   the minimal functionality an implementation must provide. The trait is now
   implemented for ROS 2.

   The crates containing the ROS 2 implementation have been renamed accordingly
   within `integrations/ros2`.

   | old                                          | new / equivalent                                 |
   | -------------------------------------------- | ------------------------------------------------ |
   | `iceoryx2-gateway`                           | `iceoryx2-link`, `iceoryx2-link-gateway`         |
   | `iceoryx2-gateway-backend`                   | `iceoryx2-link-backend`, `iceoryx2-link-adapter` |
   | `iceoryx2-gateway-conformance-tests`         | `iceoryx2-link-conformance-tests`                |
   | `iceoryx2-gateway-testing`                   | `iceoryx2-link-testing`                          |
   | `iceoryx2-integrations-ros2-gateway-backend` | `iceoryx2-integrations-ros2-link-adapter`        |
   | `iceoryx2-integrations-ros2-gateway-cli`     | `iceoryx2-integrations-ros2-link-gateway-cli`    |

   The API to create a gateway has been changed to fit the new architecture.
   A gateway is now instantiated with an adapter implementation, a mapping and
   a translator. The instantiated gateway is then passed to a link to connect
   it to services visible to a provided node. The link tracks local services
   itself.

    ```rust
    // old
    use iceoryx2_gateway::Gateway;
    use iceoryx2_integrations_ros2_gateway_backend::{PlainStructTranslator, PrefixMapping, Ros2Backend};

    let gateway_config = iceoryx2_gateway::Config::default();
    let iceoryx_config = iceoryx2::config::Config::default();
    let backend_config = iceoryx2_integrations_ros2_gateway_backend::Config::default();

    let mut gateway = Gateway::<ipc::Service, Ros2Backend<ipc::Service, PrefixMapping, PlainStructTranslator>>::new()
        .gateway_config(gateway_config)
        .iceoryx_config(iceoryx_config)
        .backend_config(backend_config)
        .mapping(PrefixMapping::new(allowlist))
        .polled()
        .create()?;

    gateway.discover()?;
    gateway.propagate()?;
    let services = gateway.bridged_services();

    // new
    use iceoryx2_integrations_ros2_link_adapter::{
        Config as AdapterConfig, PlainStructTranslator, PrefixMapping, Ros2Adapter,
    };
    use iceoryx2_link::Link;
    use iceoryx2_link_gateway::Gateway;

    let config = iceoryx2::config::Config::default();
    let node = NodeBuilder::new().config(&config).create::<ipc::Service>()?;
    let adapter = Ros2Adapter::new(&AdapterConfig::default())?;

    let mut link = Link::new(
        node,
        Gateway::new(adapter, PrefixMapping::new(allowlist), PlainStructTranslator),
    );

    link.discover()?;
    link.propagate()?;
    let bridges = link.bridges();
    ```

<!-- markdownlint-enable MD013 -->
