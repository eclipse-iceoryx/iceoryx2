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

use core::error::Error;

use crate::Frame;

/// Carries the frames of one service in both directions.
pub trait Channel {
    type Error: Error;

    /// Sends one frame to every peer on the channel.
    fn send(&mut self, frame: Frame<'_>) -> Result<(), Self::Error>;

    /// Hands the next pending frame to `on_frame`, or returns `None` if
    /// nothing is pending.
    fn receive<R>(&mut self, on_frame: impl FnOnce(&[u8]) -> R) -> Result<Option<R>, Self::Error>;
}
