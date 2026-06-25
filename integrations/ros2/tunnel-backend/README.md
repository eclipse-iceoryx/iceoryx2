# iceoryx2-integrations-ros2-tunnel-backend

> [!IMPORTANT]
> Under active development.

Tunnel backend connecting native iceoryx2 applications with ROS 2 nodes,
implemented on the
[r2r_rcl](https://github.com/sequenceplanner/r2r/tree/master/r2r_rcl)
bindings to `rcl`.

Verified with Jazzy (`rmw_fastrtps_cpp`) and Humble (`rmw_cyclonedds_cpp`).

## Status

| Relay             | Send (iceoryx2 → ROS 2) | Receive (ROS 2 → iceoryx2) |
|-------------------|-------------------------|----------------------------|
| Publish-subscribe | ✅ Implemented          | 🚧 In progress             |
| Event             | ➖ N/A                  | ➖ N/A                     |

| Capability                            | Status         |
|---------------------------------------|----------------|
| Static discovery (configured topics)  | ✅ Implemented |
| Dynamic discovery (ROS 2 graph)       | 🚧 In progress |
| Topic & QoS mapping                   | 🚧 In progress |
| Passthrough mode (CDR payloads as-is) | 🚧 In progress |
| Translation mode (CDR transcoded)     | 🚧 In progress |
| CI integration                        | 🚧 In progress |

✅ Implemented &nbsp;·&nbsp; 🚧 In progress &nbsp;·&nbsp; ➖ N/A (no ROS 2 equivalent)

## Building

The crate is a standalone workspace linking against `rcl`, so it needs a
sourced ROS 2 environment at build and run time — either the development
distrobox (see [`../README.md`](../README.md)) or any environment with
`setup.bash` sourced:

```bash
source /opt/ros/<distro>/setup.bash   # e.g. jazzy, humble
cargo build                           # the backend
cargo build --examples                # plus the tunnel runners
```

## Running

An example is provided to run the tunnel in polled mode:

```bash
cargo run --example tunnel_polled
```
