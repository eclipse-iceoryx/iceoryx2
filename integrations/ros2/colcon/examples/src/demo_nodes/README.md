# demo_nodes_iceoryx2

Minimal `iceoryx2` applications built as a ROS 2 package with colcon.
Illustrates how to package `iceoryx2` applications as `ament_cargo` packages.

## Building

Building `iceoryx2` applications as ROS 2 packages requires the cargo-aware
colcon build tools. First, install them:

```bash
pip install colcon-cargo colcon-ros-cargo
cargo install cargo-ament-build
```

Inside a container, such as the development distrobox, the tools can also be
installed with `just setup integrations-ros2` for convenience.

The package can be built with the plain `colcon` commands or, for convenience,
with a `just` recipe. In the following, `<distro>` is your ROS 2 distribution
(`jazzy` or `humble`).

With the plain commands, from the example workspace
(`integrations/ros2/colcon/examples`):

```bash
source /opt/ros/<distro>/setup.bash
colcon build --packages-select demo_nodes_iceoryx2
source install/setup.bash
```

With `just`:

```bash
source /opt/ros/<distro>/setup.bash
just setup integrations-ros2-examples
# From the repository root
source integrations/ros2/target/<distro>/colcon/examples/install/setup.bash
```

## Running

Open two terminals, source the install space of the example workspace in each,
and execute the following commands.

### Terminal 1

```bash
ros2 run demo_nodes_iceoryx2 subscriber
```

### Terminal 2

```bash
ros2 run demo_nodes_iceoryx2 publisher
```
