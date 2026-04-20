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

//! Example: receive a DMA-BUF via DmaBufSubscriber and map it with
//! MappedDmaBuf::read.
//!
//! Run: cargo run --example dmabuf-subscriber --features "dma-buf"

#[cfg(not(all(target_os = "linux", feature = "dma-buf")))]
fn main() {
    eprintln!("This example requires Linux and the dma-buf feature.");
}

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
fn main() -> Result<(), Box<dyn core::error::Error>> {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::DmaBufSubscriber;

    const SERVICE: &str = "mos4/dmabuf-example";
    const POLL_INTERVAL_MS: u64 = 10;
    const TIMEOUT_SECS: u64 = 5;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct FrameMeta {
        width: u32,
        height: u32,
    }

    let mut subscriber =
        DmaBufSubscriber::<iceoryx2::service::ipc::Service, FrameMeta>::create(SERVICE)?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(TIMEOUT_SECS);

    loop {
        if std::time::Instant::now() > deadline {
            return Err("timeout: no frame received".into());
        }
        match subscriber.recv()? {
            Some((meta, buf)) => {
                println!("Received frame: {meta:?}");
                let mapped = buf.memory_map()?;
                mapped.read(
                    |data, _: Option<()>| {
                        println!(
                            "  first byte = 0x{:02X}, total bytes = {}",
                            data[0],
                            data.len()
                        );
                        Ok(())
                    },
                    None,
                )?;
                break;
            }
            None => {
                std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS));
            }
        }
    }

    Ok(())
}
