# Example Workspace

This directory contains a colcon workspace for applications demonstrating
integration between ROS 2 and iceoryx2. These applications use the Rust
message crates of the [message workspace](../messages/README.md), included
through the [`ros-env`](https://github.com/ros2-rust/ros-env) crate.

All commands assume a **sourced ROS 2 environment**, e.g. inside the development
distrobox (see [../../README.md](../../README.md)), and are run from this
directory.

## Building

```bash
just setup integrations-ros2-examples
```

This builds the examples on top of the [message workspace](../messages/README.md)
into `../../target/<distro>/colcon/examples`. Source its `install/setup.bash`
to run them with `ros2 run`.
