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

* A sourced ROS 2 distribution
* The Rust message crates generated in the
  [message workspace](colcon/messages/README.md), for the tests and the
  [gateway examples](examples/README.md)
* The cargo-aware colcon build tools, for the
  [colcon examples](colcon/examples/README.md)

The linked READMEs describe how to set them up.

Inside a container, such as the development distrobox, all prerequisites can be
set up with a single `just` recipe for convenience:

```bash
source /opt/ros/<distro>/setup.bash # if not using the distrobox
just setup integrations-ros2
```

It installs the colcon build tools, generates the message crates and builds
the colcon examples, placing everything in `integrations/ros2/target/<distro>`.

## Building and Testing

The workspace can be built and tested with plain `cargo` commands or, for
convenience, with `just` recipes. Both require the prerequisites above.

With the plain commands, from the repository root:

```bash
source /opt/ros/<distro>/setup.bash
source integrations/ros2/colcon/messages/install/setup.bash
cargo build --manifest-path integrations/ros2/Cargo.toml --workspace
cargo nextest run --manifest-path integrations/ros2/Cargo.toml --workspace
```

With `just`:

```bash
just build integrations-ros2
just test integrations-ros2
```

The recipes source the message crates automatically from the install space
`just setup integrations-ros2` creates in `integrations/ros2/target/<distro>`.

In addition, the `just` recipes provide convenience commands to run the
end-to-end tests:

```bash
just test-e2e integrations-ros2
just test-e2e integrations-ros2-examples
```

## Examples

* [`examples`](examples/README.md) shows how to propagate services to and from
  ROS 2 via a gateway.
* [`colcon/examples`](colcon/examples/README.md) shows how to build `iceoryx2`
  applications using colcon.
