# iox2-tunnel-ros2

CLI running the tunnel that bridges iceoryx2 services and ROS 2 topics.

## Coupling to ROS 2 workspaces

The binary links `rcl` at build time and loads message typesupport
libraries at runtime, so a **sourced ROS 2 environment is required both to
build and to run it**. The built binary belongs to the distribution it was
built against; rebuild after switching distributions.

## Usage

From the repository, in a sourced shell (e.g. the development distrobox,
see [../README.md](../README.md)):

```bash
cargo run -p iceoryx2-integrations-ros2-tunnel-cli -- --help
```
