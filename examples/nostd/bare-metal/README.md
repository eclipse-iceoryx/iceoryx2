# Bare Metal Examples

> [!NOTE]
> Bare metal support is currently in early-stage development.
>
> Whilst `iceoryx2` can now be run on such platforms, it has not yet been
> optimized for resource-constrained platforms with limited memory.

In these examples we highlight the ability to deploy applications developed
against `iceoryx2` to any platform, including those without an operating
system, without the need to modify the application.

## Installation Instructions

These examples are in isolated Cargo workspaces. Building and running must be
done from within the example directories.

The `armv7r-none-eabihf` target is used to demonstrate functionality as it is
an R-Core that can be emulated in QEMU relatively easily. To be able to build
them, ensure that this target is installed on your host:

```console
rustup target add armv7r-none-eabihf
```

To be able to run the examples, additionally ensure `qemu-system-arm` is
installed on your host. Instructions for installation vary so look up the
relevant instructions for your development host.

## Build Instructions

The examples can be built with the `semihosting` feature set to see output
when emulating them on QEMU. Builds without this feature enabled provide the
template for implementing output functionality using platform-specific
capabilities.

To build and run with `semihosting` enabled, specify the feature when running
the example:

```console
cargo run --example bare_metal_nostd_publish_subscribe --features semihosting
```
