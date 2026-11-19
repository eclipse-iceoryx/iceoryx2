# Flatbuffer Translator

Utilizes a reflection-based translator from flatbuffers to the CDR format
used by ROS 2. The translator expects flatbuffer layouts matching the layouts
of the ROS 2 messages. Flatbuffer reflection and ROS 2 introspection
typesupports are leveraged to construct a model for translation when services
are discovered, which is then consulted during the hot path.

| Service       | Root type                        | Topic           | ROS 2 type                      |
| ------------- | -------------------------------- | --------------- | ------------------------------- |
| `JointState`  | `sensor_msgs.msg.JointState`     | `/joint_states` | `sensor_msgs/msg/JointState`    |
| `PoseStamped` | `geometry_msgs.msg.PoseStamped`  | `/pose`         | `geometry_msgs/msg/PoseStamped` |

* `schemas/` holds the `.fbs` schemas used by the example. Rust bindings and
  the binary `.bfbs` are generated from them with `flatc` before building.
  The binary schema is stored along with services and used for translation.
* `mapping.toml` pairs the services with the topics.
* `joint_state_publisher` and `joint_state_subscriber` publish and subscribe
  on the service `JointState`.
* `pose_stamped_publisher` and `pose_stamped_subscriber` publish and subscribe
  on the service `PoseStamped`.

The translator pairs a schema with the ROS 2 message named by its root type,
e.g. `joint_state.fbs` with `sensor_msgs/msg/JointState`:

```text
namespace builtin_interfaces.msg;

struct Time {
    sec: int32;
    nanosec: uint32;
}

namespace std_msgs.msg;

table Header {
    stamp: builtin_interfaces.msg.Time;
    frame_id: string;
}

namespace sensor_msgs.msg;

table JointState {
    header: std_msgs.msg.Header;
    name: [string];
    position: [double];
    velocity: [double];
    effort: [double];
}

root_type JointState;
```

## Prerequisites

To use the examples, first install the FlatBuffers package.

```sh
# Arch Linux
pacman -S flatbuffers

# Debian/Ubuntu
apt install libflatbuffers-dev
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
binary schemas from the `.fbs` files. The applications bake the binary schemas
into their binaries and the gateway looks them up in the directory given as its
schema path.

```sh
flatc -o integrations/ros2/examples/flatbuffer_translator/schemas --rust --schema --binary \
    integrations/ros2/examples/flatbuffer_translator/schemas/joint_state.fbs \
    integrations/ros2/examples/flatbuffer_translator/schemas/pose_stamped.fbs
```

In addition, the `just` recipes provide a convenience command to generate them
for every example:

```bash
just setup integrations-ros2
```

### Build

Build the examples and the gateway CLI:

```sh
cargo build --manifest-path integrations/ros2/Cargo.toml --examples
cargo build --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2
```

## Running

Run each command below in its own terminal. Each terminal must be sourced as
for building.

### Outbound

From `iceoryx2` to ROS 2.

#### Terminal 1: gateway

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
    --static-mapping integrations/ros2/examples/flatbuffer_translator/mapping.toml \
    --translator Flatbuffer \
    --schema-path integrations/ros2/examples/flatbuffer_translator/schemas
```

#### Terminal 2: `iceoryx2` publisher

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_joint_state_publisher
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
cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
    --static-mapping integrations/ros2/examples/flatbuffer_translator/mapping.toml \
    --translator Flatbuffer \
    --schema-path integrations/ros2/examples/flatbuffer_translator/schemas
```

#### Terminal 2: `iceoryx2` subscriber

```sh
cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_pose_stamped_subscriber
```

#### Terminal 3: ROS 2 publisher

```sh
ros2 topic pub -r 1 /pose geometry_msgs/msg/PoseStamped "{header: {frame_id: world}, pose: {position: {x: 1.0}, orientation: {w: 1.0}}}"
```

## Adding a Type

1. Write a schema whose root type names the ROS 2 message.
2. Generate its `.bfbs` into the schema path.
3. Add the service to `mapping.toml` with the payload type
   `iox2::Flatbuffer<root type>`.

The gateway warns at startup when the payload type of a mapped service does
not correspond to its ROS 2 type.

With the prefix mapping, `mapping.toml` is not needed. A service named
`ros2://topics/<topic>` is paired with the topic `/<topic>` and the ROS 2
type is taken from the schema's root type.
