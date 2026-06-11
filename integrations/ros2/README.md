# ROS 2 Integrations

> [!IMPORTANT]
> Under active development.

## Development Environment

Development is done inside a [distrobox](https://distrobox.it/) providing
ROS 2, so no ROS installation is needed on the host. One box per supported
ROS 2 distribution is defined declaratively in a `distrobox-<distro>.ini`
manifest:

| ROS 2 Distribution | RMW                  | Manifest               | Container                           |
|--------------------|----------------------|------------------------|-------------------------------------|
| Jazzy              | `rmw_fastrtps_cpp`   | `distrobox-jazzy.ini`  | `iceoryx2-integrations-ros2-jazzy`  |
| Humble             | `rmw_cyclonedds_cpp` | `distrobox-humble.ini` | `iceoryx2-integrations-ros2-humble` |

Replicate and enter a box with the manifest of the desired distribution, e.g.:

```bash
distrobox assemble create --file integrations/ros2/distrobox-jazzy.ini
distrobox enter iceoryx2-integrations-ros2-jazzy
```

Requirements on the host: `distrobox` and a container runtime (`podman` or
`docker`). Recreate a box after changing its manifest by re-running the
`assemble` command.

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

Verify the setup from two shells inside the box:

```bash
ros2 run demo_nodes_cpp talker
ros2 run demo_nodes_cpp listener
```
