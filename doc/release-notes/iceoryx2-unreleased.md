<!-- markdownlint-disable MD013 The new format requires longer lines -->

# iceoryx2 v?.?.?

## [v?.?.?](https://github.com/eclipse-iceoryx/iceoryx2/tree/v?.?.?)

[Full Changelog](https://github.com/eclipse-iceoryx/iceoryx2/compare/v?.?.?...v?.?.?)

### Features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1896](https://github.com/eclipse-iceoryx/iceoryx2/issues/1896) Add flatbuffer translation example for the ROS 2 gateway
* [#2011](https://github.com/eclipse-iceoryx/iceoryx2/issues/2011) Add a publish-subscribe latency benchmark with a FlatBuffers payload
* [#2031](https://github.com/eclipse-iceoryx/iceoryx2/issues/2031) Notify the event service named after a service when the link delivered samples to it
* [#2078](https://github.com/eclipse-iceoryx/iceoryx2/issues/2078) Allocate the sample reference counters of a grown data segment only when the data segment grows
* [#2082](https://github.com/eclipse-iceoryx/iceoryx2/issues/2082) Keep the contents of a static storage in memory and close its file
* [#2084](https://github.com/eclipse-iceoryx/iceoryx2/issues/2084) Stop iterating a slot map after its last element
* [#2094](https://github.com/eclipse-iceoryx/iceoryx2/issues/2094) Add a process-local trigger concept to `iceoryx2-cal` with an implementation based on a socket pair
* [#2098](https://github.com/eclipse-iceoryx/iceoryx2/issues/2098) Close the file descriptor of a dynamic storage once its shared memory is mapped
* [#2114](https://github.com/eclipse-iceoryx/iceoryx2/issues/2114) Truncate a loaned slice sample so that only its first elements are sent

### Bugfixes

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#1515](https://github.com/eclipse-iceoryx/iceoryx2/issues/1515) Update dependencies in zenoh: rustls-webpki from 0.103.13 to 0.103.16, webpki-roots and webpki-roots-certs from 1.0.7 to 1.0.10
* [#2018](https://github.com/eclipse-iceoryx/iceoryx2/issues/2018) Return `InvalidListenerKey` instead of panicking when notifying with a listener key whose index exceeds the service capacity
* [#2028](https://github.com/eclipse-iceoryx/iceoryx2/issues/2028) Support services with a custom payload alignment in the link
* [#2044](https://github.com/eclipse-iceoryx/iceoryx2/issues/2044) Respect the order of `PATH` when looking up CLI commands
* [#2045](https://github.com/eclipse-iceoryx/iceoryx2/issues/2045) List a CLI command only once when it is found in multiple search paths
* [#2050](https://github.com/eclipse-iceoryx/iceoryx2/issues/2050) Find CLI commands in the folder of the running binary and prefer them over the ones in `PATH`
* [#2062](https://github.com/eclipse-iceoryx/iceoryx2/issues/2062) Grant DELETE rights to the file owner in the Windows DACL
* [#2094](https://github.com/eclipse-iceoryx/iceoryx2/issues/2094) Fail a send to a socket pair on Windows when its peer is gone, like on POSIX
* [#2124](https://github.com/eclipse-iceoryx/iceoryx2/issues/2124) Remove clone impl from RelocatablePointer
* [#2137](https://github.com/eclipse-iceoryx/iceoryx2/issues/2137) Fix typo in `BackpressureAction` enum
* [#2142](https://github.com/eclipse-iceoryx/iceoryx2/issues/2142) Initialize the node counter of the global management segment when the segment is created

### Refactoring

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#818](https://github.com/eclipse-iceoryx/iceoryx2/issues/818) Factor the sample transcoding out of the gateway's publish-subscribe relay
* [#1977](https://github.com/eclipse-iceoryx/iceoryx2/issues/1977) Restructure the ROS 2 gateway into a link gateway with a ROS 2 adapter
* [#1977](https://github.com/eclipse-iceoryx/iceoryx2/issues/1977) Remove the former gateway core in `iceoryx2-gateway/`, superseded by the link crates
* [#1977](https://github.com/eclipse-iceoryx/iceoryx2/issues/1977) Tidy up implementation of the link's discovery state
* [#2010](https://github.com/eclipse-iceoryx/iceoryx2/issues/2010) Receive bytes from a tunnel directly into loaned samples

### Workflow

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#2057](https://github.com/eclipse-iceoryx/iceoryx2/issues/2057) Migrate Bazel Rust builds to rules_rs; support Bazel 7.5.0 or newer with compatibility patches
* [#2063](https://github.com/eclipse-iceoryx/iceoryx2/issues/2063) Add testing function to remove all resources under a specific config
* [#2068](https://github.com/eclipse-iceoryx/iceoryx2/issues/2068) Add `just` script to orechestrate execution of end-to-end tests
* [#2131](https://github.com/eclipse-iceoryx/iceoryx2/issues/2131) Forward interrupt errors to end-user API

### New API features

<!--
    NOTE: Add new entries sorted by issue number to minimize the possibility of
    conflicts when merging.
-->

* [#2046](https://github.com/eclipse-iceoryx/iceoryx2/issues/2046) Add abstraction for named pipes to `iceoryx2-bb-posix` and concept for event-triggering
* [#2076](https://github.com/eclipse-iceoryx/iceoryx2/issues/2076) Expose the names of ports in the port details of the C and C++ bindings
* [#2108](https://github.com/eclipse-iceoryx/iceoryx2/issues/2108) Extend Python bindings with an API to serialize the iceoryx2 config to toml

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

2. The `WaitSetBuilder` can no longer be directly instantiated. First a `Node`
   must be created and this can return a `WaitSetBuilder`

    ```rust
    // old
    let waitset = WaitSetBuilder::new().create::<ipc::Service>()>?;

    // new
    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let waitset = node.waitset_builder().create()?;
    ```

3. The `BackpressureAction` enum had a typo in `FollowBackpressureyStrategy`

    ```rust
    // old
    let bp_action = BackpressureAction::FollowBackpressureyStrategy;

    // new
    let bp_action = BackpressureAction::FollowBackpressureStrategy;
    ```

<!-- markdownlint-enable MD013 -->
