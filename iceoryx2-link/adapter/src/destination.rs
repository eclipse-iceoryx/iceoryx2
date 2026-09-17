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

use crate::ResizeError;

/// Where a message taken from the middleware is written.
///
/// A header written before the payload may incur an additional copy of the
/// header. The payload length is required to loan an iceoryx2 sample which
/// holds the buffers for both.
pub trait Destination {
    /// The payload region, `len` bytes of the wire form.
    fn payload(&mut self, len: usize) -> Result<&mut [u8], ResizeError>;

    /// The header region, `len` bytes of the header form.
    fn header(&mut self, len: usize) -> Result<&mut [u8], ResizeError>;
}
