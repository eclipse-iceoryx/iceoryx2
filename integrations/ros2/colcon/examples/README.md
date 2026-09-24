# Example Workspace

This directory contains a colcon workspace for example `iceoryx2` applications
built as ROS 2 packages.

All commands assume a **sourced ROS 2 environment**, e.g. inside the development
distrobox (see [../../README.md](../../README.md)), and are run from this
directory.

## Building

```bash
just setup integrations-ros2-examples
```

This builds the examples into `../../target/<distro>/colcon/examples`. Source
its `install/setup.bash` to run them with `ros2 run`.
