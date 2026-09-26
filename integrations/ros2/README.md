# ROS 2 Integrations

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experiementation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

## Development Environment

Development is done inside a [distrobox](https://distrobox.it/) providing
ROS 2, so no ROS installation is needed on the host. One box per supported
ROS 2 distribution is defined in a `distrobox-<distro>.ini` manifest:

| ROS 2 Distribution | RMW                  | Manifest               | Container                           |
|--------------------|----------------------|------------------------|-------------------------------------|
| Jazzy              | `rmw_fastrtps_cpp`   | `distrobox-jazzy.ini`  | `iceoryx2-integrations-ros2-jazzy`  |
| Humble             | `rmw_cyclonedds_cpp` | `distrobox-humble.ini` | `iceoryx2-integrations-ros2-humble` |

In the following, `<distro>` is your ROS 2 distribution (`jazzy` or `humble`).

With `podman` or `docker` installed on the host, replicate and enter a box with
the manifest of the desired distribution:

```bash
distrobox assemble create --file integrations/ros2/distrobox-<distro>.ini
distrobox enter iceoryx2-integrations-ros2-<distro>
```

Each box provides:

* The ROS 2 distribution (official `ros:<distro>` base image) with the
  default RMW listed above
* Build tooling for the crates (`build-essential`, `pkg-config`,
  `libclang-dev` for bindgen)
* `demo_nodes_cpp` as ready-made ROS 2 peers

Notes:

* Your host `$HOME` is shared, so the host Rust toolchain works inside the
  box unchanged.
* `/tmp` is also shared, so a host tmux server is reachable from inside the
  box. Run the in-box tmux on its own socket instead:

  ```bash
  tmux -L ros2          # new server + session on a dedicated socket
  tmux -L ros2 attach   # reattach (every tmux command needs -L ros2)
  ```

After creating and entering the container, the following can be run from
separate terminals to verify ROS 2 is properly set up:

```bash
ros2 run demo_nodes_cpp talker
ros2 run demo_nodes_cpp listener
```

## Prerequisites

The integrations require:

* A sourced, up-to-date ROS 2 distribution, or an older one with the Rust
  message crates generated in the [message workspace](colcon/messages/README.md)
* The cargo-aware colcon build tools, for the
  [colcon examples](colcon/examples/README.md)

The associated READMEs describe how to set them up.

Inside a container, such as the development distrobox, the colcon build tools
can be installed with a `just` recipe for convenience:

```bash
just setup integrations-ros2
```

## Building and Testing

The workspace can be built and tested with plain `cargo` commands or, for
convenience, with `just` recipes. Both require the prerequisites above.

With the plain commands, from the repository root:

```bash
source /opt/ros/<distro>/setup.bash
cargo build --manifest-path integrations/ros2/Cargo.toml --workspace
cargo nextest run --manifest-path integrations/ros2/Cargo.toml --workspace
```

With `just`:

```bash
just build integrations-ros2
just test integrations-ros2
```

On older installations, build the message crates in the
[message workspace](colcon/messages/README.md) first. The plain commands then
also need its install space sourced, while the `just` recipes source it
automatically.

In addition, the `just` recipes provide convenience commands to run the
end-to-end tests:

```bash
just test-e2e integrations-ros2

just setup integrations-ros2-examples
just test-e2e integrations-ros2-examples
```

## Examples

| Name                                         | Description                                                                                    |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| [gateway examples](examples/README.md)       | `iceoryx2` applications whose services are propagated to ROS 2 via a gateway.                  |
| [colcon examples](colcon/examples/README.md) | `iceoryx2` applications packaged as ROS 2 packages built with colcon and run with `ros2 run`. |

