# iceoryx2-integrations-ros2-link-adapter

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

The ROS 2 adapter of the link's gateway, implemented on the
[r2r_rcl](https://github.com/sequenceplanner/r2r/tree/master/r2r_rcl)
bindings to `rcl`.

Verified with Jazzy (`rmw_fastrtps_cpp`) and Humble (`rmw_cyclonedds_cpp`).

## Coupling to ROS 2 workspaces

The crate links `rcl` at build time and loads message typesupport
libraries at runtime, so a **sourced ROS 2 environment is required both to
build and to run it**. A build is tied to the distribution it was built
against. Rebuild if switching distributions.

## Mappings

A mapping maps service names to topics, and service settings with QoS.

* `PrefixMapping` covers a service named `ros2://topics/<topic>` as the
  topic `/<topic>`, for the topics its allow list passes. Settings and QoS
  are approximated from each other. Intended for development and
  prototyping.
* `StaticMapping` covers only the services listed in its configuration.
  Each entry pairs one `iceoryx2` service, its name, payload type and
  settings, with one ROS 2 topic, its name, message type and QoS. The
  entries are exact, a service or topic found with other settings than its
  entry states is refused. The configuration loads from any format serde
  supports, e.g. TOML:

  ```toml
  [[mapping]]
  iceoryx2.service_name = "CmdVel"
  iceoryx2.payload_type = "geometry_msgs/msg/Twist"
  ros2.topic = "/cmd_vel"
  ros2.type = "geometry_msgs/msg/Twist"
  ```

  See [`static-mapping.example.toml`](../link-gateway-cli/static-mapping.example.toml)
  for a complete example.

## Translators

A translator translates between the samples of an `iceoryx2` service and
the messages of a ROS 2 topic.

* `PlainStructTranslator` translates samples holding the C struct rosidl
  generates for the message type, so publishers send and subscribers
  receive the struct itself. Each sample is converted to and from the CDR
  form ROS 2 sends. Message types containing pointers, such as
  strings and sequences, are not supported.
* `PassthroughTranslator` translates samples holding a byte slice named
  after the message type. Each sample holds one ROS 2 message serialized
  as CDR, exactly as ROS 2 sends it, and is not converted. So **publishers
  must serialize the message to CDR before writing it into a sample,
  and subscribers must deserialize the CDR in the samples they receive**.
  Since the messages are serialized, any message type can be supported.

## Usage

The adapter runs under a gateway, with a mapping and a translator:

```rust
use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_link_adapter::{
    AllowList, Config, PlainStructTranslator, PrefixMapping, Ros2Adapter,
};
use iceoryx2_link::Link;
use iceoryx2_link_gateway::Gateway;

let node = NodeBuilder::new().create::<ipc::Service>()?;

let mapping = PrefixMapping::new(AllowList::all());
// A static mapping instead, from its configuration:
// let config: static_mapping::Config = toml::from_str(&content)?;
// let mapping = StaticMapping::new(config)?;

let translator = PlainStructTranslator;
// The CDR bytes instead:
// let translator = PassthroughTranslator;

let adapter = Ros2Adapter::new(&Config::default())?;
// With a static mapping, to load the typesupports of the types its
// entries name eagerly on instantiation, otherwise they are loaded
// on first use:
// let adapter = Ros2Adapter::new(&Config {
//     preload_types: mapping.type_names(),
//     ..Config::default()
// })?;

let gateway = Gateway::new(adapter, mapping, translator);

let mut link = Link::new(node, gateway);
let listener = link.listener()?;

loop {
    listener.blocking_wait(|_| {})?;
    link.discover()?;
    link.propagate()?;
}
```

## CLI

The gateway is also available as a command line tool, see
[`link-gateway-cli`](../link-gateway-cli/README.md).
