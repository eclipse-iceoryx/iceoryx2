# iceoryx2-dmabuf

## Overview

`iceoryx2-dmabuf` provides a `dmabuf::Service` variant inside iceoryx2 for
kernel-owned DMA-BUF frame delivery: V4L2 ISP capture, DRM scanout, GBM
buffers, and Vulkan external memory. It uses a **parallel-construct
architecture** — a dedicated Unix-domain socket connection carries file
descriptors alongside iceoryx2 typed metadata — rather than implementing the
`iceoryx2::service::Service` trait. See design decision D5 in
[`specs/arch-dmabuf-service-variant.adoc`](specs/arch-dmabuf-service-variant.adoc).

## Quick Start

### Lowest level — raw connection (no metadata)

```rust
use iceoryx2_dmabuf::connection::linux::Linux;

let mut pub_conn = Linux::open_publisher("camera/frames")?;
pub_conn.send_with_fd(borrowed_fd, byte_len)?;

let mut sub_conn = Linux::open_subscriber("camera/frames")?;
if let Some((owned_fd, len)) = sub_conn.recv_with_fd()? {
    // use fd
}
```

### Service level — typed metadata + raw fd

```rust
use iceoryx2_dmabuf::service::Service;

let factory = Service::open_or_create::<u64>("camera/frames")?;
let mut publisher = factory.publisher_builder().create()?;
publisher.publish(frame_id, borrowed_fd, byte_len)?;

let mut subscriber = factory.subscriber_builder().create()?;
if let Some((frame_id, owned_fd, len)) = subscriber.receive()? {
    // use fd and frame_id
}
```

### Typed level — `dma-buf` feature (recommended)

```rust
use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};

let mut publisher = DmaBufPublisher::<u64>::create("camera/frames")?;
publisher.publish(frame_id, &dma_buf)?;

let mut subscriber = DmaBufSubscriber::<u64>::create("camera/frames")?;
if let Some((frame_id, buf)) = subscriber.receive()? {
    let mapped = buf.memory_map()?;
    mapped.read(|data, _: Option<()>| { /* read bytes */ Ok(()) }, None)?;
}
```

See [`examples/publish_subscribe_dmabuf_service/`](examples/publish_subscribe_dmabuf_service/)
for runnable publisher and subscriber binaries.

## Cargo Features

| Feature | Description | Platform |
|---|---|---|
| `default` | `std` | All |
| `std` | iceoryx2 std support | All |
| `dma-buf` | typed `DmaBufPublisher` / `DmaBufSubscriber` over upstream `dma-buf` 0.5 | Linux only |
| `peercred` | `SO_PEERCRED` UID check on UDS accept | Linux only |

## Architecture

Design rationale and wire protocol are documented in:

- [`specs/arch-dmabuf-service-variant.adoc`](specs/arch-dmabuf-service-variant.adoc) — decisions D1–D8
- [`specs/spec-dmabuf-service-variant.adoc`](specs/spec-dmabuf-service-variant.adoc) — detailed specification

## Migration from Sidecar Prototype (PR #1572)

| Sidecar | Service variant (this branch) |
|---|---|
| `FdSidecarPublisher<S, Meta>` | `DmaBufServicePublisher<Meta>` (or `DmaBufPublisher<Meta>` typed) |
| `FdSidecarSubscriber<S, Meta>` | `DmaBufServiceSubscriber<Meta>` |
| `FdSidecarToken` user-header | NONE — token in wire frame |
| `FdSidecarError` | `ServiceError` + `connection::Error` |
| `BackChannel` / `BufferReleased` | `release(token)` / `recv_release_ack()` on ports |

## Out of Scope

- Multi-planar formats (YUV420 separate planes)
- Async transport
- `sync_file` GPU fence integration
- Windows / non-Linux platforms

## License

Apache-2.0 OR MIT
