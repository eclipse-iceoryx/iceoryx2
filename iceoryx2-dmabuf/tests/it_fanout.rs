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

//! Integration test: 1-producer, 3-consumer fan-out over 100 frames.
//!
//! * Producer sends 100 frames of distinct `memfd` regions.
//! * 3 subscribers each receive all 100 frames.
//! * At frame 50 subscriber[0] is dropped; subscribers[1] and [2] continue
//!   through frame 100 unaffected.
//! * Each received fd is `fstat`'d and its inode is compared to the
//!   producer's inode for that frame — verifying identity, not just liveness.
//!
//! Linux-only; compiled out on non-Linux targets.

#[cfg(target_os = "linux")]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{FdSidecarPublisher, FdSidecarSubscriber};
    use rustix::fs::{MemfdFlags, memfd_create};
    use std::os::fd::{AsFd as _, AsRawFd as _, FromRawFd as _, OwnedFd};
    use std::time::Duration;

    const N_FRAMES: u64 = 100;
    const DROP_AFTER: u64 = 50;
    const SETTLE_MS: u64 = 20;
    const SERVICE: &str = "fanout-three-test";

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        frame: u64,
    }

    /// Create a `memfd_create`'d fd and return it together with its inode.
    fn make_frame(seq: u64) -> (OwnedFd, u64) {
        use std::io::Write as _;

        let name = format!("fanout-frame-{seq}");
        let fd = memfd_create(name.as_str(), MemfdFlags::CLOEXEC).expect("memfd_create failed");
        {
            let raw = fd.as_fd().as_raw_fd();
            // SAFETY: fd is valid and owned; ManuallyDrop prevents double-close.
            let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(raw) });
            let payload = seq.to_le_bytes();
            file.write_all(&payload).expect("write_all failed");
        }
        let stat = rustix::fs::fstat(&fd).expect("fstat failed");
        let ino = stat.st_ino;
        (fd, ino)
    }

    /// Poll a subscriber for up to 500 ms until it receives a frame.
    fn recv_frame(
        sub: &mut FdSidecarSubscriber<iceoryx2::service::ipc::Service, Meta>,
    ) -> Option<(Meta, OwnedFd)> {
        const POLL_MS: u64 = 10;
        const TIMEOUT_MS: u64 = 500;

        let deadline = std::time::Instant::now() + Duration::from_millis(TIMEOUT_MS);
        while std::time::Instant::now() < deadline {
            match sub.recv() {
                Ok(Some(pair)) => return Some(pair),
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(POLL_MS));
                }
                Err(e) => {
                    panic!("recv error: {e:?}");
                }
            }
        }
        None
    }

    #[test]
    fn one_producer_three_consumers_100_frames() {
        let unique = format!("{}-{}", SERVICE, std::process::id());

        let mut pub_ = FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(&unique)
            .expect("FdSidecarPublisher::create failed");

        let mut sub0 =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&unique)
                .expect("sub0 create failed");
        let mut sub1 =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&unique)
                .expect("sub1 create failed");
        let mut sub2 =
            FdSidecarSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(&unique)
                .expect("sub2 create failed");

        // Allow all subscribers to connect to the publisher socket.
        std::thread::sleep(Duration::from_millis(SETTLE_MS));

        // Track which inodes each subscriber received.
        let mut inodes_sub0: Vec<u64> = Vec::new();
        let mut inodes_sub1: Vec<u64> = Vec::new();
        let mut inodes_sub2: Vec<u64> = Vec::new();

        // Publish frames 1..=DROP_AFTER; all three subscribers receive.
        for seq in 1..=DROP_AFTER {
            let (fd, pub_ino) = make_frame(seq);

            pub_.send(Meta { frame: seq }, fd).expect("send failed");

            std::thread::sleep(Duration::from_millis(SETTLE_MS));

            let (_, fd0) =
                recv_frame(&mut sub0).unwrap_or_else(|| panic!("sub0 did not recv frame {seq}"));
            let stat0 = rustix::fs::fstat(&fd0).expect("fstat fd0");
            assert_eq!(stat0.st_ino, pub_ino, "sub0 inode mismatch at frame {seq}");
            inodes_sub0.push(stat0.st_ino);

            let (_, fd1) =
                recv_frame(&mut sub1).unwrap_or_else(|| panic!("sub1 did not recv frame {seq}"));
            let stat1 = rustix::fs::fstat(&fd1).expect("fstat fd1");
            assert_eq!(stat1.st_ino, pub_ino, "sub1 inode mismatch at frame {seq}");
            inodes_sub1.push(stat1.st_ino);

            let (_, fd2) =
                recv_frame(&mut sub2).unwrap_or_else(|| panic!("sub2 did not recv frame {seq}"));
            let stat2 = rustix::fs::fstat(&fd2).expect("fstat fd2");
            assert_eq!(stat2.st_ino, pub_ino, "sub2 inode mismatch at frame {seq}");
            inodes_sub2.push(stat2.st_ino);
        }

        // Drop sub0 at frame 50.
        drop(sub0);

        // Publish frames DROP_AFTER+1..=N_FRAMES; only sub1 and sub2 receive.
        for seq in (DROP_AFTER + 1)..=N_FRAMES {
            let (fd, pub_ino) = make_frame(seq);

            pub_.send(Meta { frame: seq }, fd).expect("send failed");

            std::thread::sleep(Duration::from_millis(SETTLE_MS));

            let (_, fd1) =
                recv_frame(&mut sub1).unwrap_or_else(|| panic!("sub1 did not recv frame {seq}"));
            let stat1 = rustix::fs::fstat(&fd1).expect("fstat fd1");
            assert_eq!(stat1.st_ino, pub_ino, "sub1 inode mismatch at frame {seq}");
            inodes_sub1.push(stat1.st_ino);

            let (_, fd2) =
                recv_frame(&mut sub2).unwrap_or_else(|| panic!("sub2 did not recv frame {seq}"));
            let stat2 = rustix::fs::fstat(&fd2).expect("fstat fd2");
            assert_eq!(stat2.st_ino, pub_ino, "sub2 inode mismatch at frame {seq}");
            inodes_sub2.push(stat2.st_ino);
        }

        assert_eq!(
            inodes_sub0.len(),
            DROP_AFTER as usize,
            "sub0 should have received exactly {DROP_AFTER} frames"
        );
        assert_eq!(
            inodes_sub1.len(),
            N_FRAMES as usize,
            "sub1 should have received exactly {N_FRAMES} frames"
        );
        assert_eq!(
            inodes_sub2.len(),
            N_FRAMES as usize,
            "sub2 should have received exactly {N_FRAMES} frames"
        );

        // All inodes in each list must be distinct (each frame has its own memfd).
        let mut sorted = inodes_sub1.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            N_FRAMES as usize,
            "sub1 received duplicate inodes"
        );

        let mut sorted2 = inodes_sub2.clone();
        sorted2.sort_unstable();
        sorted2.dedup();
        assert_eq!(
            sorted2.len(),
            N_FRAMES as usize,
            "sub2 received duplicate inodes"
        );
    }
}
