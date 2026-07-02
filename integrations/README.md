# `iceoryx2` Integrations

This directory holds integrations that connect `iceoryx2` to external
middlewares and frameworks.

Each subdirectory is a self-contained integration for one external system,
maintained as its **own cargo workspace** rather than as a member of the main
workspace. This keeps their dependencies isolated from the core.

## Available Integrations

| Integration | Description                                                                          | `just` scope         | Details                            |
|-------------|-------------------------------------------------------------------------------------|----------------------|------------------------------------|
| Zenoh       | Connect `iceoryx2` systems across hosts and networks via [Zenoh](https://zenoh.io). | `integrations-zenoh` | [`zenoh/`](zenoh)                  |
| ROS 2       | Interoperate between `iceoryx2` and the [ROS 2](https://www.ros.org) ecosystem.     | `integrations-ros2`  | [`ros2/README.md`](ros2/README.md) |

## Building & Testing

Integrations are driven through the repository's `just` recipes from the
repository root. The recipe resolves the right workspace:

```bash
just build integrations-zenoh
just test integrations-zenoh
```

The ROS 2 integration additionally requires a sourced ROS 2 environment —
see [`ros2/README.md`](ros2/README.md) for the development distrobox that
provides one:

```bash
just build integrations-ros2
just test integrations-ros2
```
