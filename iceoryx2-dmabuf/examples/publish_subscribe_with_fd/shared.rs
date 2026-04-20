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

//
//! Shared type definitions for the publish_subscribe_with_fd examples.
//!
//! Both `publisher.rs` and `subscriber.rs` duplicate this struct locally
//! because Cargo examples cannot share a common module file without a
//! workspace-level crate.  This file documents the canonical layout.

/// Frame metadata transmitted alongside each DMA-BUF fd.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FrameMeta {
    /// Frame width in pixels.
    pub width: u32,
    /// Frame height in pixels.
    pub height: u32,
    /// FourCC pixel format code (e.g. `0x3231_5659` = `YV12`).
    pub fourcc: u32,
    /// Monotonically increasing frame sequence number.
    pub seq: u64,
}
// Safety: FrameMeta is `repr(C)`, `Copy`, and contains no padding bytes of
// undefined value.
unsafe impl iceoryx2::prelude::ZeroCopySend for FrameMeta {}
