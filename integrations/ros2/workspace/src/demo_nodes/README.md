# demo_nodes_iceoryx2

Native iceoryx2 example nodes bridged to ROS 2 by the tunnel, one
publisher/subscriber pair per combination of service-to-topic mapping and
payload translator:

| Binary pair (`*_{publisher,subscriber}`)     | Mapping                                                                | Translator    |
|----------------------------------------------|------------------------------------------------------------------------|---------------|
| `prefix_mapping_passthrough_translator_*`    | `ros2://topics/` name prefix                                           | passthrough   |

Applications using the passthrough translator serialize paylaods to
CDR themselves. The CDR-serialized payloads are shared with the tunnel (and
other applications) via shared memory. Other applications must deserialize
the applications themselves. The tunnel in passthrough mode however can
forward the bytes directly to ROS 2.

See [../../README.md](../../README.md) for the build setup common to all examples.

## Building

Ensure the pre-requisites described in [../../README.md](../../README.md)
are done and run, from the workspace root:

```bash
colcon build --packages-select demo_nodes_iceoryx2
```

## Running

### Prefix mapping + passthrough translator

Outbound (iceoryx2 → ROS 2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 prefix_mapping_passthrough_translator_publisher
# in other shells:
#   cargo run -p iceoryx2-integrations-ros2-tunnel-cli
#   ros2 run demo_nodes_cpp listener
```

Inbound (ROS 2 → iceoryx2), where the topic must be allowlisted for
wire-side discovery, as the tunnel only bridges ROS 2 topics it is
explicitly told about instead of mirroring the entire graph:

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 prefix_mapping_passthrough_translator_subscriber
# in other shells:
#   cargo run -p iceoryx2-integrations-ros2-tunnel-cli -- --topic /chatter:std_msgs/msg/String
#   ros2 run demo_nodes_cpp talker
```
