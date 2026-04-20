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

//! Unit tests for DmaBufPublisher / DmaBufSubscriber (feature = "dma-buf").

#[cfg(all(target_os = "linux", feature = "dma-buf"))]
mod linux_dma_buf_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::{DmaBufPublisher, DmaBufSubscriber};

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    /// DmaBufPublisher::create succeeds on Linux when the dma-buf feature is
    /// enabled and constructs the inner FdSidecarPublisher.
    #[test]
    fn dmabuf_publisher_create_linux() {
        let result = DmaBufPublisher::<iceoryx2::service::ipc::Service, Meta>::create(
            "unit-dmabuf-pub-create",
        );
        assert!(
            result.is_ok(),
            "DmaBufPublisher::create must succeed on Linux: {:?}",
            result.err()
        );
    }

    /// DmaBufSubscriber::create succeeds on Linux after a publisher has bound
    /// the side-channel UDS (the subscriber connects; it cannot stand alone).
    #[test]
    fn dmabuf_subscriber_create_linux() {
        let pub_result = DmaBufPublisher::<iceoryx2::service::ipc::Service, Meta>::create(
            "unit-dmabuf-sub-create",
        );
        assert!(
            pub_result.is_ok(),
            "DmaBufPublisher::create must succeed on Linux: {:?}",
            pub_result.err()
        );
        // Keep the publisher bound by holding the Result; dropping pub_result
        // would drop the publisher and unbind the UDS.
        let _publisher = pub_result;

        let sub_result = DmaBufSubscriber::<iceoryx2::service::ipc::Service, Meta>::create(
            "unit-dmabuf-sub-create",
        );
        assert!(
            sub_result.is_ok(),
            "DmaBufSubscriber::create must succeed on Linux after publisher exists: {:?}",
            sub_result.err()
        );
    }
}

#[cfg(not(target_os = "linux"))]
mod non_linux_tests {
    use iceoryx2::prelude::ZeroCopySend;
    use iceoryx2_dmabuf::FdSidecarError;

    #[derive(Debug, Clone, Copy, ZeroCopySend)]
    #[repr(C)]
    struct Meta {
        seq: u64,
    }

    /// On non-Linux stubs, DmaBufPublisher::create returns UnsupportedPlatform.
    /// (This test only runs without the dma-buf feature because the type is
    ///  cfg-gated on linux + feature; on non-Linux we verify via FdSidecar*.)
    #[test]
    fn create_returns_unsupported_platform_on_non_linux() {
        let result =
            iceoryx2_dmabuf::FdSidecarPublisher::<iceoryx2::service::ipc::Service, Meta>::create(
                "unit-dmabuf-non-linux",
            );
        assert!(
            matches!(result, Err(FdSidecarError::UnsupportedPlatform)),
            "expected UnsupportedPlatform on non-Linux"
        );
    }
}
