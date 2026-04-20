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

//! Integration test: full DMA-BUF heap roundtrip.
//!
//! Allocates 4 KiB via dma_heap::Heap (system heap), publishes via
//! DmaBufPublisher, receives via DmaBufSubscriber, maps via MappedDmaBuf::read,
//! asserts a known byte pattern.
//!
//! Skipped (not panicked) when /dev/dma_heap/system is absent.

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{DmaBuf, DmaBufPublisher, DmaBufSubscriber};
    use std::os::fd::AsFd as _;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    const SERVICE: &str = "it-dmabuf-heap-roundtrip";
    const MAGIC: u8 = 0xEF;
    const BUF_SIZE: usize = 4096;
    const SETTLE_MS: u64 = 20;
    const DMA_HEAP_PATH: &str = "/dev/dma_heap/system";

    #[test]
    fn dmabuf_heap_roundtrip() -> Result<(), Box<dyn core::error::Error>> {
        // Skip gracefully when the dma-heap device is unavailable.
        if !std::path::Path::new(DMA_HEAP_PATH).exists() {
            eprintln!("SKIP: {DMA_HEAP_PATH} not present — dma-heap kernel support required");
            return Ok(());
        }

        let heap = dma_heap::Heap::new(dma_heap::HeapKind::System)?;

        // allocate() returns OwnedFd; dup so we have one for writing and one for sending.
        let owned_fd = heap.allocate(BUF_SIZE)?;
        let dup_fd = owned_fd.as_fd().try_clone_to_owned()?;

        // Write magic bytes into the write copy.
        let write_buf = DmaBuf::from(owned_fd);
        let mut mapped = write_buf.memory_map()?;
        mapped.write(
            |data, _: Option<()>| {
                data.iter_mut().for_each(|b| *b = MAGIC);
                Ok(())
            },
            None,
        )?;
        drop(mapped); // unmap before sending

        // Use the dup'd fd for sending.
        let send_buf = DmaBuf::from(dup_fd);

        let mut pub_ = DmaBufPublisher::<iceoryx2::service::ipc::Service, Meta>::create(SERVICE)?;
        let mut sub_ = DmaBufSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(SERVICE)?;

        std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

        pub_.send(Meta { seq: 99 }, &send_buf)?;

        std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

        let received = sub_.recv()?;
        let (meta, received_buf) = received.ok_or("recv returned None")?;
        assert_eq!(meta.seq, 99);

        let mapped = received_buf.memory_map()?;
        mapped.read(
            |data, _: Option<()>| {
                assert!(
                    data.iter().all(|&b| b == MAGIC),
                    "byte pattern mismatch after heap roundtrip"
                );
                Ok(())
            },
            None,
        )?;

        Ok(())
    }
}
