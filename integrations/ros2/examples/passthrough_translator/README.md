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

## How to Run

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
