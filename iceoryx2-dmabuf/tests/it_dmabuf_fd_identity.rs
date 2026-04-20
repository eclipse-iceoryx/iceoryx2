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

//! Integration test: fd inode is preserved through the DmaBuf wrap.
//!
//! Publisher holds a memfd wrapped as DmaBuf; after roundtrip the subscriber
//! receives a DmaBuf whose underlying fd has the same (st_dev, st_ino).

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
mod linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{DmaBuf, DmaBufPublisher, DmaBufSubscriber};
    use rustix::fs::{MemfdFlags, fstat, memfd_create};
    use std::os::fd::AsFd as _;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    const SERVICE: &str = "it-dmabuf-fd-identity";
    const SETTLE_MS: u64 = 20;

    #[test]
    fn fd_identity_preserved_through_wrap() -> Result<(), Box<dyn core::error::Error>> {
        let owned_fd = memfd_create("dmabuf-identity-test", MemfdFlags::CLOEXEC)?;

        // Obtain (st_dev, st_ino) before publishing.
        let pub_stat = fstat(&owned_fd)?;
        let (pub_dev, pub_ino) = (pub_stat.st_dev, pub_stat.st_ino);

        let buf = DmaBuf::from(owned_fd);

        let mut pub_ = DmaBufPublisher::<iceoryx2::service::ipc::Service, Meta>::create(SERVICE)?;
        let mut sub_ = DmaBufSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(SERVICE)?;

        std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

        pub_.send(Meta { seq: 42 }, &buf)?;

        std::thread::sleep(std::time::Duration::from_millis(SETTLE_MS));

        let received = sub_.recv()?;
        let (_, received_buf) = received.ok_or("recv returned None")?;

        // The received DmaBuf's fd must point to the same kernel inode.
        let sub_stat = fstat(received_buf.as_fd())?;

        assert_eq!(
            (pub_dev, pub_ino),
            (sub_stat.st_dev, sub_stat.st_ino),
            "inode mismatch: publisher ({pub_dev}, {pub_ino}) != subscriber ({}, {})",
            sub_stat.st_dev,
            sub_stat.st_ino,
        );

        Ok(())
    }
}
