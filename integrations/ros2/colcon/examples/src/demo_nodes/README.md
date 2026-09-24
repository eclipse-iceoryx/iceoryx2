# demo_nodes_iceoryx2

Minimal `iceoryx2` applications built as a ROS 2 package with colcon.
Illustrates how to package `iceoryx2` applications as `ament_cargo` packages.

## Building

Ensure the pre-requisites described in [ros2/colcon/examples/README.md](../../README.md)
are done and run, from the workspace root:

```bash
colcon build --packages-select demo_nodes_iceoryx2
```

## Running

Open two terminals and execute the following commands.

### Terminal 1

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 subscriber
```

### Terminal 2

```bash
source <workspace>/install/setup.bash
ros2 run demo_nodes_iceoryx2 publisher
```
