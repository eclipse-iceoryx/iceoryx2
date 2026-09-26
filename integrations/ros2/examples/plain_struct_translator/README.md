# Plain Struct Translator

Applications whose services carry the C struct rosidl generates for a ROS 2
message (`geometry_msgs/msg/Twist`) which the gateway CLI's plain-struct
translator serializes to CDR at the boundary.

One publisher and subscriber pair per mapping:

* `prefix_mapping_*` on the service `ros2://topics/cmd_vel`, which the
  prefix mapping pairs with the topic `/cmd_vel`.
* `static_mapping_*` on the service `CmdVel`, which
  [`static_mapping.toml`](static_mapping.toml) pairs with the topic
  `/cmd_vel`.

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
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --translator PlainStruct --ros-header
```

### Terminal 2

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_publisher
```

### Terminal 3

```sh
ros2 topic echo /cmd_vel geometry_msgs/msg/Twist
```

Inbound, from ROS 2 to `iceoryx2`. With the prefix mapping, the topic needs
to be explicitly allowed:

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --allow /cmd_vel --translator PlainStruct --ros-header
```

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_subscriber
```

```sh
ros2 topic pub -r 1 /cmd_vel geometry_msgs/msg/Twist "{linear: {x: 0.5}}"
```

With the static mapping, the gateway uses a mapping configuration:

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --static-mapping integrations/ros2/examples/plain_struct_translator/static_mapping.toml --translator PlainStruct --ros-header
```
