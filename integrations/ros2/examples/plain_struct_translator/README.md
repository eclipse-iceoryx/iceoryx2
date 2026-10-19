# Plain Struct Translator

Applications whose services carry the C structs rosidl generates for ROS 2
messages (`geometry_msgs/msg/Twist`, `example_interfaces/srv/AddTwoInts`)
which the gateway CLI's plain-struct translator serializes to CDR at the
boundary.

* `prefix_mapping_publisher` and `prefix_mapping_subscriber` on the service
  `ros2://topics/cmd_vel`, which the prefix mapping pairs with the topic
  `/cmd_vel`.
* `prefix_mapping_client` and `prefix_mapping_server` on the service
  `ros2://services/add_two_ints`, which the prefix mapping pairs with the
  ROS 2 service `/add_two_ints`.
* `static_mapping_publisher` and `static_mapping_subscriber` on the service
  `CmdVel`, which [`static_mapping.toml`](static_mapping.toml) pairs with
  the topic `/cmd_vel`.
* `static_mapping_client` and `static_mapping_server` on the service
  `AddTwoInts`, which [`static_mapping.toml`](static_mapping.toml) pairs
  with the ROS 2 service `/add_two_ints`.

## Building

Building the examples requires your ROS 2 distribution to be sourced. On
up-to-date installations, it also provides the Rust crates of its message
packages. In the following, `<distro>` is your ROS 2 distribution (`jazzy` or
`humble`).

```sh
source /opt/ros/<distro>/setup.bash
```

On older installations that do not ship the Rust message crates, build them
from source as described in the [message workspace
README](../../colcon/messages/README.md) and additionally source its install
space.

Then build the examples and the gateway CLI:

```sh
cargo build --manifest-path integrations/ros2/Cargo.toml --examples
cargo build --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2
```

## Running

Run each command below in its own terminal. Each terminal must be sourced as
for building.

### Prefix Mapping

The prefix mapping pairs services named `ros2://topics/<topic>` with the topic
`/<topic>` and services named `ros2://services/<service>` with the ROS 2
service `/<service>`.

#### Publish-Subscribe

##### Outbound

From `iceoryx2` to ROS 2.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --translator PlainStruct --ros-header
```

###### Terminal 2: `iceoryx2` publisher

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_publisher
```

###### Terminal 3: ROS 2 subscriber

```sh
ros2 topic echo /cmd_vel geometry_msgs/msg/Twist
```

##### Inbound

From ROS 2 to `iceoryx2`. Allow only the example's topic with `--allow`, as the
gateway otherwise mirrors every topic it discovers in the ROS 2 graph.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --allow /cmd_vel --translator PlainStruct --ros-header
```

###### Terminal 2: `iceoryx2` subscriber

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_subscriber
```

###### Terminal 3: ROS 2 publisher

```sh
ros2 topic pub -r 1 /cmd_vel geometry_msgs/msg/Twist "{linear: {x: 0.5}}"
```

#### Request-Response

##### Outbound

From `iceoryx2` to ROS 2, an `iceoryx2` client calls a ROS 2 server. Allow
only the example's service with `--allow`, as the gateway otherwise mirrors
every service it discovers in the ROS 2 graph.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --allow /add_two_ints --translator PlainStruct
```

###### Terminal 2: ROS 2 server

```sh
ros2 run demo_nodes_cpp add_two_ints_server
```

###### Terminal 3: `iceoryx2` client

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_client
```

##### Inbound

From ROS 2 to `iceoryx2`, a ROS 2 client calls an `iceoryx2` server. Allow
only the example's service with `--allow`.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --allow /add_two_ints --translator PlainStruct
```

###### Terminal 2: `iceoryx2` server

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_server
```

###### Terminal 3: ROS 2 client

```sh
ros2 service call /add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 3}"
```

### Static Mapping

The static mapping pairs the services and topics listed in
[`static_mapping.toml`](static_mapping.toml).

#### Publish-Subscribe

##### Outbound

From `iceoryx2` to ROS 2.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --static-mapping integrations/ros2/examples/plain_struct_translator/static_mapping.toml --translator PlainStruct --ros-header
```

###### Terminal 2: `iceoryx2` publisher

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_static_mapping_publisher
```

###### Terminal 3: ROS 2 subscriber

```sh
ros2 topic echo /cmd_vel geometry_msgs/msg/Twist
```

##### Inbound

From ROS 2 to `iceoryx2`.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --static-mapping integrations/ros2/examples/plain_struct_translator/static_mapping.toml --translator PlainStruct --ros-header
```

###### Terminal 2: `iceoryx2` subscriber

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_static_mapping_subscriber
```

###### Terminal 3: ROS 2 publisher

```sh
ros2 topic pub -r 1 /cmd_vel geometry_msgs/msg/Twist "{linear: {x: 0.5}}"
```

#### Request-Response

##### Outbound

From `iceoryx2` to ROS 2, an `iceoryx2` client calls a ROS 2 server.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --static-mapping integrations/ros2/examples/plain_struct_translator/static_mapping.toml --translator PlainStruct
```

###### Terminal 2: ROS 2 server

```sh
ros2 run demo_nodes_cpp add_two_ints_server
```

###### Terminal 3: `iceoryx2` client

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_static_mapping_client
```

##### Inbound

From ROS 2 to `iceoryx2`, a ROS 2 client calls an `iceoryx2` server.

###### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- --static-mapping integrations/ros2/examples/plain_struct_translator/static_mapping.toml --translator PlainStruct
```

###### Terminal 2: `iceoryx2` server

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_static_mapping_server
```

###### Terminal 3: ROS 2 client

```sh
ros2 service call /add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 3}"
```
