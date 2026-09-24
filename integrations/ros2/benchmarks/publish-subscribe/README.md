# benchmark-ros2-publish-subscribe

> [!IMPORTANT]
> The ROS 2 integrations are currently prototypes and require validation in
> real ROS 2 workflows. Only recommended for experiementation in development
> deployments.
>
> If encountering issues, create an issue to help us converge to stability.

Benchmark for the ROS 2 publish-subscribe messaging pattern.

The benchmark quantifies the latency between a `Publisher` sending a message and
a `Subscription` receiving it. In the setup, a bidirectional connection is
established from process `a` to `b` (topic name `a2b`) and back (topic name
`b2a`). Every participant blocks on a wait set and promptly responds upon
message reception. This process repeats `n` times, and the average latency is
subsequently computed.

Every iteration publishes the same message and takes the incoming one into a
newly allocated message, as a ROS 2 node does. Therefore the measured latency
also contains the cost of publishing and receiving a message.

The benchmark is written against `rcl` using [r2r_rcl](https://github.com/sequenceplanner/r2r/tree/master/r2r_rcl).

## Building

The crate is part of a standalone workspace linking against `rcl`, so it needs a
sourced ROS 2 environment at build and run time. This can be the development
distrobox (see [`../../README.md`](../../README.md)) or any environment with
`setup.bash` sourced. All commands are run from this directory:

```sh
source /opt/ros/<distro>/setup.bash   # e.g. jazzy, humble
cargo build --release
```

## Running

The rmw implementation is selected with `RMW_IMPLEMENTATION` and is reported
alongside every result. Only `rmw_fastrtps_cpp` is currently supported.

```sh
cargo run --bin benchmark-ros2-publish-subscribe --release -- --intraprocess-delivery full
```

For more benchmark configuration details, see

```sh
cargo run --bin benchmark-ros2-publish-subscribe --release -- --help
```
