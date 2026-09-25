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

With `podman` or `docker` installed on the host, replicate and enter a box with
the manifest of the desired distribution, e.g.:

```bash
distrobox assemble create --file integrations/ros2/distrobox-jazzy.ini
distrobox enter iceoryx2-integrations-ros2-jazzy
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

## Preparing the Workspace

The crates and examples use Rust message crates generated from the ROS 2
message definitions. They can be built with `just` scripts from the repository
root within the distrobox:

```bash
just setup integrations-ros2
```

This installs the colcon build tools, generates the message crates and
builds the examples. Everything the ros2 workspace builds is placed in
`integrations/ros2/target/<distro>`.

The setup only runs inside a container. It installs the colcon build tools
into the system Python. The script will not install anything if not within
a container.

## Building and Testing

```bash
just build integrations-ros2
just test integrations-ros2
just test-e2e integrations-ros2
just test-e2e integrations-ros2-examples
```
