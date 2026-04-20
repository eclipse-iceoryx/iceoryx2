# iceoryx2-dmabuf

DMA-BUF fd-passing over iceoryx2 pub/sub via `SCM_RIGHTS` sidecar socket.

## Overview

`iceoryx2-dmabuf` extends iceoryx2 with a side-channel for passing DMA-BUF
file descriptors alongside zero-copy SHM payloads. The publisher binds a
per-service Unix domain socket; each subscriber connects and receives the fd
via `sendmsg(2)` with `SCM_RIGHTS` ancillary data, synchronised to the
iceoryx2 message sequence number.

## Platform

- **Linux**: full implementation (`memfd_create`, `SCM_RIGHTS`, `poll`).
- **macOS / other**: stubs return `DmabufError::UnsupportedPlatform` at
  runtime; crate compiles cleanly on all targets.

## Usage

```toml
[dependencies]
iceoryx2-dmabuf = { version = "0.8", features = ["memfd", "peercred"] }
```

## Why this crate

iceoryx2's typed SHM pool cannot transfer kernel-owned DMA-BUF allocations
(V4L2 ISP frames, DRM scanout buffers, Vulkan external-memory exports). These
allocations are referenced by file descriptors that are meaningless outside the
originating process; cross-process transfer requires `SCM_RIGHTS` over a Unix
domain socket. `iceoryx2-dmabuf` adds a thin sidecar channel that delivers fds
in lockstep with the normal iceoryx2 metadata flow, preserving zero-copy
semantics end-to-end.

## Features

| Feature | Description | Platform |
|---------|-------------|----------|
| `memfd` | Enable `memfd_create`-based fd allocation helpers | Linux only |
| `peercred` | Verify subscriber PID via `SO_PEERCRED` before fd delivery | Linux only |

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or
[MIT license](../LICENSE-MIT) at your option (same as iceoryx2).


