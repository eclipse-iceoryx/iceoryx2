# iceoryx2-integrations-ros2-tunnel-backend

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experiementation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

Tunnel backend connecting native iceoryx2 applications with ROS 2 nodes,
implemented on the
[r2r_rcl](https://github.com/sequenceplanner/r2r/tree/master/r2r_rcl)
bindings to `rcl`.

Verified with Jazzy (`rmw_fastrtps_cpp`) and Humble (`rmw_cyclonedds_cpp`).

## Status

| Relay             | Send (iceoryx2 → ROS 2) | Receive (ROS 2 → iceoryx2) |
|-------------------|-------------------------|----------------------------|
| Publish-subscribe | ✅ Implemented          | ✅ Implemented             |
| Event             | ➖ N/A                  | ➖ N/A                     |

| Capability                            | Status         |
|---------------------------------------|----------------|
| Static discovery (configured topics)  | ✅ Implemented |
| Dynamic discovery (ROS 2 graph)       | ✅ Implemented |
| Topic & QoS mapping                   | ✅ Implemented |
| Passthrough mode (CDR payloads as-is) | ✅ Implemented |
| Translation mode (CDR transcoded)     | ✅ Implemented |
| CI integration                        | ✅ Implemented |

✅ Implemented &nbsp;·&nbsp; 🚧 In progress &nbsp;·&nbsp; ➖ N/A (no ROS 2 equivalent)

## Building

The crate is a standalone workspace linking against `rcl`, so it needs a
sourced ROS 2 environment at build and run time. This can be the development
distrobox (see [`../README.md`](../README.md)) or any environment with
`setup.bash` sourced:

```bash
source /opt/ros/<distro>/setup.bash   # e.g. jazzy, humble
cargo build
```

## Running

The tunnel is run via the CLI in [`../tunnel-cli`](../tunnel-cli):

```bash
cargo run -p iceoryx2-integrations-ros2-tunnel-cli
```
