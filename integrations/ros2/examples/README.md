# ROS 2 Examples

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

`iceoryx2` applications whose services are propagated to ROS 2 via a gateway.

## Prerequisites

The examples use Rust message crates generated from the ROS 2 message
definitions, built as described in
[../colcon/messages/README.md](../colcon/messages/README.md). Before building
the examples, source your ROS 2 distribution and the install space of the
needed message crates. In the following, `<distro>` is your ROS 2 distribution
(`jazzy` or `humble`).

When built with the plain `colcon` commands, the install space is in the message
workspace:

```bash
source /opt/ros/<distro>/setup.bash
source integrations/ros2/colcon/messages/install/setup.bash
```

When built with `just setup integrations-ros2-messages`, it is in the target
directory instead:

```bash
source /opt/ros/<distro>/setup.bash
source integrations/ros2/target/<distro>/colcon/messages/install/setup.bash
```

Building the examples without the message crates will fails and name those that
are missing.

All commands in the examples are run from the repository root.

## Overview

| Name                                               | Description                                                                                                                     |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| [passthrough translator](passthrough_translator)   | Services carrying ROS 2 messages as CDR bytes propagated unchanged by the gateway.                                              |
| [plain struct translator](plain_struct_translator) | Services carrying the C struct of a ROS 2 message serialized at the boundary to CDR by the gateway.                             |
