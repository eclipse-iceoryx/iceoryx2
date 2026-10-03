# Flatbuffer Translator

Illustrates how to implement a translator for a system communicating
flatbuffers on `iceoryx2` services. It assumes a system designed before
deciding to integrate with ROS 2. The schemas used are specific to the system,
and are mapped to ROS 2 topic types:

| Service         | Flatbuffer            | Topic           | ROS 2 type                      |
| --------------- | --------------------- | --------------- | ------------------------------- |
| `JointReadings` | `robot.JointReadings` | `/joint_states` | `sensor_msgs/msg/JointState`    |
| `Pose`          | `robot.Pose`          | `/pose`         | `geometry_msgs/msg/PoseStamped` |

* `schemas/` holds the two `.fbs` schemas, the Rust code `flatc` generates
  from them, and their binary `.bfbs` form, which an `iceoryx2` flatbuffer
  service stores as its type definition.
* `translator/transcoders/` holds one transcoder per pair to convert between
  the flatbuffer and the CDR bytes propagated in ROS 2.
* `translator/mod.rs` holds the translator. It embeds the two `.bfbs`, which
  are compared with the schema stored in the service on discovery and provides
  the matching transcoder to use for the bytes passing between service and
  topic.
* `mapping.toml` provides the mapping used by the gateway to map service names
  to topics.
* `gateway.rs` initializes a gateway with the implemented translator.

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

Then build the examples, including the gateway:

```sh
cargo build --manifest-path integrations/ros2/Cargo.toml --examples
```

## Running

Run each command below in its own terminal. Each terminal must be sourced as
for building.

### Outbound

From `iceoryx2` to ROS 2.

#### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_gateway
```

#### Terminal 2: `iceoryx2` publisher

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_joint_readings_publisher
```

#### Terminal 3: ROS 2 subscriber

```sh
ros2 topic echo --qos-reliability reliable /joint_states sensor_msgs/msg/JointState
```

Naming the type and the reliability lets `echo` start before the topic
exists.

### Inbound

From ROS 2 to `iceoryx2`.

#### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_gateway
```

#### Terminal 2: `iceoryx2` subscriber

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_pose_subscriber
```

#### Terminal 3: ROS 2 publisher

```sh
ros2 topic pub -r 1 /pose geometry_msgs/msg/PoseStamped "{header: {frame_id: world}, pose: {position: {x: 1.0}, orientation: {w: 1.0}}}"
```

## Writing Your Own

For another type, add a schema, a transcoder for its pair, and a branch in
the translator.

> [!NOTE]
> A payload type is identified by the bytes of its schema, so types defined in
> one shared schema file cannot be told apart. Each type needs its own schema.
