# Specialized Flatbuffer Translator

Illustrates how to implement a translator for a system communicating
flatbuffers on `iceoryx2` services. It assumes a system designed before
deciding to integrate with ROS 2. The schemas used are specific to the system,
and are mapped to ROS 2 topic types:

| `iceoryx2` service | Flatbuffer            | ROS 2 topic     | ROS 2 type                      |
| ------------------ | --------------------- | --------------- | ------------------------------- |
| `JointReadings`    | `robot.JointReadings` | `/joint_states` | `sensor_msgs/msg/JointState`    |
| `Pose`             | `robot.Pose`          | `/pose`         | `geometry_msgs/msg/PoseStamped` |

* `schemas/` holds the `.fbs` schemas used by the example. Rust bindings and
  the binary `.bfbs` are generated from them with `flatc` before building.
  The binary schema is stored along with `iceoryx2` services and used for
  translation.
* `translator/transcoders/` holds one transcoder per pair to convert between
  the flatbuffer and the CDR bytes propagated in ROS 2.
* `translator/mod.rs` holds the translator. It embeds the two `.bfbs`, which
  are compared with the schema stored in the `iceoryx2` service on discovery
  and provides the matching transcoder to use for the bytes passing between
  the `iceoryx2` service and the ROS 2 topic.
* `mapping.toml` pairs the `iceoryx2` services with the ROS 2 topics.
* `gateway.rs` initializes a gateway with the implemented translator.

## Prerequisites

The examples need `flatc` of the `flatbuffers` crate version pinned in the
workspace `Cargo.toml`. The `just` recipes install it into `~/.local/bin`:

```sh
just setup flatc
```

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

### Generate Bindings and Binary Schemas

Before building the example, we need to generate the Rust bindings and the
binary schemas from the `.fbs` files. They must be placed at the location
specified since the translator takes the binary schemas from there to bake
into the binary.

```sh
flatc -o integrations/ros2/examples/specialized_flatbuffer_translator/schemas --rust --schema --binary \
    integrations/ros2/examples/specialized_flatbuffer_translator/schemas/joint_readings.fbs \
    integrations/ros2/examples/specialized_flatbuffer_translator/schemas/pose.fbs
```

In addition, the `just` recipes provide a convenience command to generate them
for every example:

```bash
just setup integrations-ros2
```

### Build

Build the examples, including the gateway:

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
cargo run --manifest-path integrations/ros2/Cargo.toml --example specialized_flatbuffer_gateway
```

#### Terminal 2: `iceoryx2` publisher

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example specialized_flatbuffer_joint_readings_publisher
```

#### Terminal 3: ROS 2 subscriber

```sh
ros2 topic echo --qos-reliability reliable /joint_states sensor_msgs/msg/JointState
```

Naming the type and the reliability lets `echo` start before the ROS 2 topic
exists.

### Inbound

From ROS 2 to `iceoryx2`.

#### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example specialized_flatbuffer_gateway
```

#### Terminal 2: `iceoryx2` subscriber

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example specialized_flatbuffer_pose_subscriber
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
