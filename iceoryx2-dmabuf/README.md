# iceoryx2-dmabuf

SCM_RIGHTS fd sidecar + typed DMA-BUF wrapper for iceoryx2 pub/sub.

## Overview

`iceoryx2-dmabuf` extends iceoryx2 with two clearly scoped APIs:

* **Transport** — `FdSidecarPublisher` / `FdSidecarSubscriber`: pass any `OwnedFd`
  (memfd, eventfd, pipe end, DMA-BUF fd) out-of-band via `SCM_RIGHTS` over a Unix
  domain socket, synchronised to the iceoryx2 metadata sequence number.
* **Payload** — `DmaBufPublisher` / `DmaBufSubscriber` (feature `dma-buf`): typed
  wrappers that accept / yield `dma_buf::DmaBuf`, giving subscribers safe CPU access
  via `MappedDmaBuf::read/write/readwrite` (which wrap `DMA_BUF_IOCTL_SYNC`).

## Decision matrix

| You have… | Use |
|-----------|-----|
| Arbitrary `OwnedFd` (memfd, eventfd, pipe end) with no CPU-sync concern | `FdSidecarPublisher` / `FdSidecarSubscriber` |
| `dma_buf::DmaBuf` (from `dma-heap`, V4L2, GBM, Vulkan) | `DmaBufPublisher` / `DmaBufSubscriber` |
| No DmaBuf yet — want allocation | `dma_heap::Heap::allocate` → `DmaBuf::from(fd)` → `DmaBufPublisher` |

## Platform

- **Linux**: full implementation. `DmaBuf*` types require the `dma-buf` feature.
- **macOS / other**: stubs return `FdSidecarError::UnsupportedPlatform` at runtime;
  crate compiles cleanly on all targets with `--no-default-features`.

## Transport layer

```toml
[dependencies]
iceoryx2-dmabuf = { version = "0.9", features = ["memfd"] }
```

```rust
use iceoryx2::service::ipc;
use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};

let mut pub_ = FdSidecarPublisher::<ipc::Service, u64>::create("my-service")?;
let mut sub_ = FdSidecarSubscriber::<ipc::Service, u64>::create("my-service")?;
// pub_.send(meta, owned_fd)?;
// let (meta, owned_fd) = sub_.recv()?.unwrap();
```

## Payload layer (DMA-BUF)

```toml
[dependencies]
iceoryx2-dmabuf = { version = "0.9", features = ["dma-buf"] }
dma-heap = "0.3"   # only needed for allocation
```

```rust
use iceoryx2::service::ipc;
use iceoryx2_dmabuf::{DmaBuf, DmaBufPublisher, DmaBufSubscriber};
use std::os::fd::AsFd as _;

// Allocate a DMA-BUF buffer (heap.allocate returns OwnedFd)
let heap = dma_heap::Heap::new(dma_heap::HeapKind::System)?;
let owned_fd = heap.allocate(4096)?;
let send_buf = DmaBuf::from(owned_fd);

let mut pub_ = DmaBufPublisher::<ipc::Service, u64>::create("my-service")?;
pub_.send(42u64, &send_buf)?;

let mut sub_ = DmaBufSubscriber::<ipc::Service, u64>::create("my-service")?;
let (meta, buf) = sub_.recv()?.unwrap();
let mapped = buf.memory_map()?;
mapped.read(|data, _: Option<()>| {
    println!("first byte: 0x{:02X}", data[0]);
    Ok(())
}, None)?;
```

## Features

| Feature | Description | Platform |
|---------|-------------|----------|
| `memfd` | `memfd_create`-based helpers for tests and examples | Linux only |
| `peercred` | `SO_PEERCRED` UID verification before fd delivery | Linux only |
| `dma-buf` | `DmaBufPublisher` / `DmaBufSubscriber` typed wrapper | Linux only |
| `test-utils` | Injection APIs for error-path integration tests | Linux only |

## License

Licensed under either of [Apache License, Version 2.0](../LICENSE-APACHE) or
[MIT license](../LICENSE-MIT) at your option (same as iceoryx2).
