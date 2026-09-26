# Message Workspace

This directory contains a colcon workspace generating the Rust message crates
of the ROS 2 message packages the integration uses from source. Applications
include the crates through the [`ros-env`](https://github.com/ros2-rust/ros-env)
crate, which provides the message crates of the sourced environment.

Up-to-date installations of the ROS 2 distributions ship these crates with
their message packages, so sourcing the distribution is usually sufficient.
On older installations that do not ship them, this workspace generates them
from the message definitions imported by `<distro>.repos`.

## Building

The crates can either be built with the plain `colcon` commands or with a
provided `just` recipe for convenience. The install space of the built crates
needs to be sourced before building or running any applications that depend on
them. In the following, `<distro>` is your ROS 2 distribution (`jazzy` or
`humble`).

With the plain commands, from this directory:

```bash
source /opt/ros/<distro>/setup.bash
mkdir -p src
vcs import src < <distro>.repos
colcon build --packages-up-to std_msgs geometry_msgs rosidl_generator_rs
source install/setup.bash
```

With `just`:

```bash
source /opt/ros/<distro>/setup.bash
just setup integrations-ros2-messages
# From the repository root
source integrations/ros2/target/<distro>/colcon/messages/install/setup.bash
```
