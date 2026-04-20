// Copyright (c) 2026 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Example: allocate a DMA-BUF with dma-heap and publish it via
//! DmaBufPublisher.
//!
//! Run: cargo run --example dmabuf-publisher --features "dma-buf"

#[cfg(not(all(target_os = "linux", feature = "dma-buf")))]
fn main() {
    eprintln!("This example requires Linux and the dma-buf feature.");
    eprintln!("Run with: cargo run --example dmabuf-publisher --features dma-buf");
}

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
fn main() -> Result<(), Box<dyn core::error::Error>> {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{DmaBuf, DmaBufPublisher};
    use std::os::fd::AsFd as _;

    const SERVICE: &str = "mos4/dmabuf-example";
    const BUF_SIZE: usize = 4096;
    const MAGIC: u8 = 0xAB;
    const SETTLE_MS: u64 = 100;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct FrameMeta {
        width: u32,
        height: u32,
    }

    // Allocate a DMA-BUF from the system heap.
    // dma_heap::Heap::allocate returns OwnedFd; dup so we keep one for sending.
    let heap = dma_heap::Heap::new(dma_heap::HeapKind::System)?;
    let owned_fd = heap.allocate(BUF_SIZE)?;
    let dup_fd = owned_fd.as_fd().try_clone_to_owned()?;

    // Write a known byte pattern via memory_map.
    let write_buf = DmaBuf::from(owned_fd);
    let mut mapped = write_buf.memory_map()?;
    mapped.write(
        |data, _: Option<()>| {
            data.iter_mut().for_each(|b| *b = MAGIC);
            Ok(())
        },
        None,
    )?;
    drop(mapped);

    // Use the dup'd fd for sending.
    let send_buf = DmaBuf::from(dup_fd);

    let mut publisher =
        DmaBufPublisher::<iceoryx2::service::ipc::Service, FrameMeta>::create(SERVICE)?;

    // Give the subscriber time to connect.
    std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

    publisher.send(
        FrameMeta {
            width: 64,
            height: 64,
        },
        &send_buf,
    )?;
    println!("Published DMA-BUF frame (service={SERVICE})");

    // Keep alive for the subscriber to receive.
    std::thread::sleep(std::time::Duration::from_millis(500));

    Ok(())
}
