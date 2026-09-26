# Passthrough Translator

Applications whose services carry a ROS 2 message (`std_msgs/msg/String`)
as CDR bytes which the gateway CLI's passthrough translator propagates
unchanged. The `iceoryx2` applications take on the responsibility of
(de)serialization.

One publisher and subscriber pair per mapping:

* `prefix_mapping_*` on the service `ros2://topics/chatter`, which the
  prefix mapping pairs with the topic `/chatter`.
* `static_mapping_*` on the service `Chatter`, which
  [`static_mapping.toml`](static_mapping.toml) pairs with the topic
  `/chatter`.

## Building

First, build the Rust message crates as described in the [message workspace
README](../../colcon/messages/README.md). Building the examples requires your
ROS 2 distribution and the install space of the message crates to be sourced.

In the following, `<distro>` is your ROS 2 distribution (`jazzy` or `humble`).

When the message crates were built with the plain `colcon` commands:

```sh
source /opt/ros/<distro>/setup.bash
source integrations/ros2/colcon/messages/install/setup.bash
```

When they were built with `just setup integrations-ros2-messages`:

```sh
source /opt/ros/<distro>/setup.bash
source integrations/ros2/target/<distro>/colcon/messages/install/setup.bash
```

Then build the examples and the gateway CLI:

```sh
cargo build --manifest-path integrations/ros2/Cargo.toml --examples
cargo build --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2
```

## Running

Open a terminal for each command, sourced as for building, and execute the
following commands.

Outbound, from `iceoryx2` to ROS 2, with the prefix mapping:

### Terminal 1

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --ros-header
```

### Terminal 2

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example passthrough_prefix_mapping_publisher
```

### Terminal 3

```sh
ros2 topic echo /chatter std_msgs/msg/String
```

Inbound, from ROS 2 to `iceoryx2`. With the prefix mapping, the topic needs
to be explicitly allowed:

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --allow /chatter --ros-header
```

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example passthrough_prefix_mapping_subscriber
```

```sh
ros2 topic pub -r 1 /chatter std_msgs/msg/String "{data: hello}"
```

With the static mapping, the gateway uses a mapping configuration:

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --static-mapping integrations/ros2/examples/passthrough_translator/static_mapping.toml --ros-header
```
