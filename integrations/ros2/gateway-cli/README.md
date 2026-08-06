# iox2-gateway-ros2

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experiementation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

CLI running the gateway that bridges iceoryx2 services and ROS 2 topics.

## Coupling to ROS 2 workspaces

The binary links `rcl` at build time and loads message typesupport
libraries at runtime, so a **sourced ROS 2 environment is required both to
build and to run it**. The built binary belongs to the distribution it was
built against; rebuild after switching distributions.

## Usage

From the repository, in a sourced shell (e.g. the development distrobox,
see [../README.md](../README.md)):

```bash
cargo run --bin iox2-gateway-ros2 -- --help
```
