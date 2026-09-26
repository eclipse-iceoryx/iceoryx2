# Example Workspace

This directory contains a colcon workspace of `iceoryx2` applications packaged
as ROS 2 (`ament_cargo`) packages, showing how they are built with colcon and
run with `ros2 run`. These exist to illustrate the setup for when wanting
to integrate with the ROS 2 build system.

## Prerequisites

Building `iceoryx2` applications as ROS 2 packages requires the cargo-aware
colcon build tools:

```bash
pip install colcon-cargo colcon-ros-cargo
cargo install cargo-ament-build
```

Inside a container, such as the development distrobox, the tools can also be
installed with `just setup integrations-ros2-tools` for convenience.

## Overview

| Name                         | Description                                                |
| ---------------------------- | ---------------------------------------------------------- |
| [demo nodes](src/demo_nodes) | Minimal publisher and subscriber built as a ROS 2 package. |
