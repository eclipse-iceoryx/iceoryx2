// SPDX-License-Identifier: Apache-2.0 OR MIT
#![cfg(all(target_os = "linux", feature = "dma-buf"))]

//! Integration test: full DMA-BUF heap allocation round-trip.
//!
//! Allocates 4 KiB via `dma_heap::Heap` (system heap), writes a known byte
//! pattern publisher-side via `MappedDmaBuf::write`, publishes via
//! `DmaBufPublisher`, receives via `DmaBufSubscriber`, then reads and asserts
//! the pattern via `MappedDmaBuf::read` (which pairs `DMA_BUF_IOCTL_SYNC`
//! start/end on cache-incoherent SoCs).
//!
//! Skipped (not failed) when `/dev/dma_heap/system` is absent — expected on
//! developer machines and CI environments without the dma-heap kernel module.

use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};
use std::os::fd::AsFd as _;
use std::time::Duration;

const DMA_HEAP_PATH: &str = "/dev/dma_heap/system";
const BUF_SIZE: usize = 4096;
const MAGIC: u8 = 0xAB;
const SERVICE: &str = "dmabuf/test/heap";
const DEADLINE: Duration = Duration::from_millis(500);

/// Poll until predicate returns true, or deadline elapses (last check included).
/// Synchronous test-only helper; sleep is intentional, not inside any async context.
fn wait_until<F: FnMut() -> bool>(deadline: Duration, mut f: F) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < deadline {
        if f() {
            return true;
        }
        // Intentional: synchronous test polling helper, not in production or async code.
        std::thread::sleep(Duration::from_millis(2));
    }
    f()
}

#[test]
fn heap_allocation_roundtrip_with_sync_ioctl() {
    // Skip cleanly when the dma-heap device is unavailable (developer machine, CI).
    if !std::path::Path::new(DMA_HEAP_PATH).exists() {
        println!("skip: {DMA_HEAP_PATH} not present — dma-heap kernel support required");
        return;
    }

    let heap = dma_heap::Heap::new(dma_heap::HeapKind::System).expect("open /dev/dma_heap/system");

    // Allocate BUF_SIZE bytes; returns an OwnedFd.
    let owned_fd = heap.allocate(BUF_SIZE).expect("heap allocate");

    // Duplicate: one fd for writing the seed pattern, one for publishing.
    let dup_fd = owned_fd
        .as_fd()
        .try_clone_to_owned()
        .expect("dup fd for publishing");

    // Seed a known pattern publisher-side via IOCTL_SYNC start/end.
    let write_buf: dma_buf::DmaBuf = owned_fd.into();
    let mut mapped = write_buf.memory_map().expect("memory_map write side");
    mapped
        .write(
            |data, _: Option<()>| {
                data[..64].copy_from_slice(&[MAGIC; 64]);
                Ok(())
            },
            None,
        )
        .expect("mapped write");
    drop(mapped);

    // Use the duplicated fd as the buffer to publish.
    let send_buf: dma_buf::DmaBuf = dup_fd.into();

    let mut pubr = DmaBufPublisher::<u64>::create(SERVICE).expect("publisher create");
    let mut subr = DmaBufSubscriber::<u64>::create(SERVICE).expect("subscriber create");

    // Allow the UDS fd-channel handshake to complete before publishing.
    // Intentional blocking settle delay in synchronous test code.
    std::thread::sleep(Duration::from_millis(50));

    pubr.publish(99_u64, &send_buf).expect("publish");

    // Poll until a sample arrives; capture it to avoid re-calling receive.
    let mut result: Option<(u64, dma_buf::DmaBuf)> = None;
    let arrived = wait_until(DEADLINE, || {
        match subr.receive().expect("receive must not error") {
            Some(pair) => {
                result = Some(pair);
                true
            }
            None => false,
        }
    });
    assert!(arrived, "no sample arrived within {DEADLINE:?}");

    let (meta, recv_buf) = result.expect("result set when arrived is true");
    assert_eq!(meta, 99_u64, "meta mismatch");

    // Verify the pattern subscriber-side via DMA_BUF_IOCTL_SYNC start/end.
    let mapped = recv_buf.memory_map().expect("memory_map subscriber side");
    mapped
        .read(
            |data, _: Option<()>| {
                for (i, &b) in data[..64].iter().enumerate() {
                    assert_eq!(
                        b, MAGIC,
                        "byte mismatch at index {i}: got {b:#04x}, expected {MAGIC:#04x}"
                    );
                }
                Ok(())
            },
            None,
        )
        .expect("mapped read");
}
