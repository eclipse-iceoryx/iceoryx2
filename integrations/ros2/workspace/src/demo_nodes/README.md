# demo_nodes_iceoryx2

Native iceoryx2 example nodes bridged to ROS 2 by the tunnel, one
publisher/subscriber pair per combination of service-to-topic mapping and
payload translator:

| Binary pair (`*_{publisher,subscriber}`)     | Mapping                                                                | Translator   |
|----------------------------------------------|------------------------------------------------------------------------|--------------|
| `prefix_mapping_passthrough_translator_*`    | `ros2://topics/` name prefix                                           | passthrough  |
| `prefix_mapping_plain_struct_translator_*`   | `ros2://topics/` name prefix                                           | plain-struct |
| `static_mapping_passthrough_translator_*`    | entries of [static_mapping_chatter.toml](static_mapping_chatter.toml)  | passthrough  |
| `static_mapping_plain_struct_translator_*`   | entries of [static_mapping_cmdvel.toml](static_mapping_cmdvel.toml)    | plain-struct |

Applications using the passthrough translator serialize paylaods to
CDR themselves. The CDR-serialized payloads are shared with the tunnel (and
other applications) via shared memory. Other applications must deserialize
the applications themselves. The tunnel in passthrough mode however can
forward the bytes directly to ROS 2.

Conversely, the plain-struct translator shares the self-contained POD structs
generated from ROS 2 messages via shared memory. The tunnel serializes these
to CDR at the boundary to ROS 2.

See [ros2/workspace/README.md](../../README.md) for the build setup common to all examples.

## Building

Ensure the pre-requisites described in [ros2/workspace/README.md](../../README.md)
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
#   cargo run --bin iox2-tunnel-ros2
#   ros2 run demo_nodes_cpp listener
```

Inbound (ROS 2 → iceoryx2), where the topic must be allowlisted for
wire-side discovery, as the tunnel only bridges ROS 2 topics it is
explicitly told about instead of mirroring the entire graph:

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 prefix_mapping_passthrough_translator_subscriber
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- --topic /chatter:std_msgs/msg/String
#   ros2 run demo_nodes_cpp talker
```

### Prefix mapping + plain-struct translator

Outbound (iceoryx2 → ROS 2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 prefix_mapping_plain_struct_translator_publisher
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- --translator PlainStruct
#   ros2 topic echo /cmd_vel
```

Inbound (ROS 2 → iceoryx2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 prefix_mapping_plain_struct_translator_subscriber
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- \
#       --topic /cmd_vel:geometry_msgs/msg/Twist \
#       --translator PlainStruct
#   ros2 topic pub -r 1 /cmd_vel geometry_msgs/msg/Twist "{linear: {x: 0.5}}"
```

### Static mapping + passthrough translator

Both directions run the tunnel on the example's mapping file, which pairs
the service `Chatter` with the topic `/chatter` and doubles as the
wire-side discovery allowlist.

Outbound (iceoryx2 → ROS 2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 static_mapping_passthrough_translator_publisher
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- --static-mapping workspace/src/demo_nodes/static_mapping_chatter.toml
#   ros2 run demo_nodes_cpp listener
```

Inbound (ROS 2 → iceoryx2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 static_mapping_passthrough_translator_subscriber
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- --static-mapping workspace/src/demo_nodes/static_mapping_chatter.toml
#   ros2 run demo_nodes_cpp talker
```

### Static mapping + plain-struct translator

Both directions run the tunnel with the plain-struct translator on a
separate mapping file, which pairs the service `CmdVel` with the topic
`/cmd_vel`. Separate, because the translator admits fixed-size types only
and fails resolution for `std_msgs/msg/String`, so the chatter entry must
not be in scope.

Outbound (iceoryx2 → ROS 2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 static_mapping_plain_struct_translator_publisher
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- \
#       --static-mapping workspace/src/demo_nodes/static_mapping_cmdvel.toml \
#       --translator PlainStruct
#   ros2 topic echo /cmd_vel
```

Inbound (ROS 2 → iceoryx2):

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 static_mapping_plain_struct_translator_subscriber
# in other shells:
#   cargo run --bin iox2-tunnel-ros2 -- \
#       --static-mapping workspace/src/demo_nodes/static_mapping_cmdvel.toml \
#       --translator PlainStruct
#   ros2 topic pub -r 1 /cmd_vel geometry_msgs/msg/Twist "{linear: {x: 0.5}}"
```
