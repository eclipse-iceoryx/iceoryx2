# Publish-Subscribe (Bare Metal)

> [!NOTE]
> Bare metal support is currently in early-stage development.
>
> Whilst `iceoryx2` can now be run on such platforms, it has not yet been
> optimized for resource-constrained platforms with limited memory.

This example illustrates the ability to use the publish-subscribe messaging
pattern in a bare metal deployment. Samples are sent and received with the
same publish-subscribe APIs used in more typical deployements to platforms
with an operating system.

Here we want to highlight the ability to deploy applications developed against
`iceoryx2` to any platform, including those without an operating system,
without the need to modify the application.

## How to Run

The example application is build for the `armv7r-none-eabihf` target and
emulated using QEMU.

Ensure the toolchain is installed:

```console
rustup target add armv7r-none-eabihf
```

And install `qemu-system-arm` on your system so you may run the example.
Instructions for installation will differ based on your operating system.

Then, build and run the example with the `semihosting` feature enabled so that
output can be provided:

```console
cargo run --features semihosting
```
