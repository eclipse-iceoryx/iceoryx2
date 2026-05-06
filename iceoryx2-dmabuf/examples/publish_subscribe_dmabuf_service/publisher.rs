// SPDX-License-Identifier: Apache-2.0 OR MIT
//! End-to-end DMA-BUF publisher.
//!
//! Allocates DMA-BUF frames from /dev/dma_heap/system and publishes them
//! over the dmabuf::Service variant of iceoryx2.
//!
//! Run alongside the matching subscriber example:
//! ```bash
//! cargo run -p iceoryx2-dmabuf --features dma-buf --example dmabuf-service-publisher
//! ```

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Frame buffer size: 1920 × 1080 × 4 bytes (BGRA8).
    const FRAME_BYTES: usize = 1920 * 1080 * 4;
    /// Target inter-frame interval in milliseconds (~30 fps).
    const FRAME_INTERVAL_MS: u64 = 33;
    /// Total number of frames to publish before exiting.
    const FRAME_COUNT: u64 = 100;

    let heap = dma_heap::Heap::new(dma_heap::HeapKind::System)?;
    let mut publisher = iceoryx2_dmabuf::DmaBufPublisher::<u64>::create("camera/frames")?;

    println!("dmabuf publisher: starting on service 'camera/frames'");
    for frame_id in 0u64..FRAME_COUNT {
        let owned_fd = heap.allocate(FRAME_BYTES)?;
        let buf: dma_buf::DmaBuf = owned_fd.into();
        publisher.publish(frame_id, &buf)?;
        println!("dmabuf publisher: sent frame {frame_id}");
        // Synchronous single-threaded example binary; blocking sleep is intentional.
        std::thread::sleep(std::time::Duration::from_millis(FRAME_INTERVAL_MS));
    }
    Ok(())
}

#[cfg(not(all(target_os = "linux", feature = "dma-buf")))]
fn main() {
    eprintln!("This example requires Linux + the 'dma-buf' Cargo feature");
    std::process::exit(1);
}
