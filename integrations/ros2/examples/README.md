# ROS 2 Examples

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experimentation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

`iceoryx2` applications whose services are propagated to ROS 2 via a gateway.

## Prerequisites

A sourced ROS 2 environment, e.g. the development distrobox (see
[../README.md](../README.md)), plus the Rust message crates the examples
use. The build fails with a message naming what is missing otherwise.

```bash
just setup integrations-ros2-messages
source integrations/ros2/target/<distro>/colcon/messages/install/setup.bash
```

All commands in the examples are run from the repository root.

## Overview

| Name                                               | Description                                                                                                                     |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| [passthrough translator](passthrough_translator)   | Services carrying ROS 2 messages as CDR bytes propagated unchanged by the gateway.                                              |
| [plain struct translator](plain_struct_translator) | Services carrying the C struct of a ROS 2 message serialized at the boundary to CDR by the gateway.                             |
