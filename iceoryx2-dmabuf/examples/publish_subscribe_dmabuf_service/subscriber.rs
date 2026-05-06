// SPDX-License-Identifier: Apache-2.0 OR MIT
//! End-to-end DMA-BUF subscriber.
//!
//! Receives frames from the matching publisher example, mmaps each frame,
//! and reads its first byte (exercises DMA_BUF_IOCTL_SYNC start/end pairing
//! via MappedDmaBuf::read on cache-incoherent SoCs).
//!
//! Run alongside the matching publisher example:
//! ```bash
//! cargo run -p iceoryx2-dmabuf --features dma-buf --example dmabuf-service-subscriber
//! ```

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// Poll interval when no sample is available (milliseconds).
    const POLL_INTERVAL_MS: u64 = 5;

    let mut subscriber = iceoryx2_dmabuf::DmaBufSubscriber::<u64>::create("camera/frames")?;

    println!("dmabuf subscriber: listening on 'camera/frames'");
    loop {
        match subscriber.receive()? {
            Some((frame_id, buf)) => {
                let mapped = buf.memory_map()?;
                mapped.read(
                    |data, _: Option<()>| {
                        println!(
                            "dmabuf subscriber: frame {frame_id} first byte 0x{:02X}",
                            data[0]
                        );
                        Ok(())
                    },
                    None,
                )?;
            }
            // Synchronous single-threaded example binary; blocking sleep is intentional.
            None => std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)),
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "dma-buf")))]
fn main() {
    eprintln!("This example requires Linux + the 'dma-buf' Cargo feature");
    std::process::exit(1);
}
