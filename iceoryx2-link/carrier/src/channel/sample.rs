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

use iceoryx2_link_backend::wire::sample::SampleBytesRef;

/// A channel to carry one service's samples between local and remote peers.
pub trait SampleChannel {
    type Error: Error;

    /// Sends the bytes of one sample to every peer connected to the channel.
    fn send(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Error>;

    /// The concatenated bytes of the next pending sample, header then payload,
    /// or `None` if nothing is pending.
    fn receive(&mut self) -> Result<Option<&[u8]>, Self::Error>;
}
