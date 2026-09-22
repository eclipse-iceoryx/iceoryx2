# Example Workspace

This directory contains a colcon workspace for applications demonstrating
integration between ROS 2 and iceoryx2. These applications use the Rust
message crates of the [message workspace](../messages/README.md), included
through the [`ros-env`](https://github.com/ros2-rust/ros-env) crate.

All commands assume a **sourced ROS 2 environment**, e.g. inside the development
distrobox (see [../../README.md](../../README.md)), and are run from this
directory.

## Prerequisites

Build the [message workspace](../messages/README.md) and source its
`install/setup.bash`.

## Building

```bash
colcon build --packages-select demo_nodes_iceoryx2
```
