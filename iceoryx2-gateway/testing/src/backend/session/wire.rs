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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use iceoryx2::service::service_hash::ServiceHash;

use super::{SendError, SessionId};

#[derive(Debug)]
pub struct Sample {
    pub header: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Envelope {
    pub(super) from: SessionId,
    pub(super) kind: Kind,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum Kind {
    Event {
        service_hash: ServiceHash,
        id: u64,
    },
    Sample {
        service_hash: ServiceHash,
        header: Vec<u8>,
        payload: Vec<u8>,
    },
}

/// Serialize an envelope into the given buffer for sending as a datagram.
pub(super) fn serialize_envelope<'a>(
    envelope: &Envelope,
    buf: &'a mut [u8],
) -> Result<&'a [u8], SendError> {
    match postcard::to_slice(envelope, buf) {
        Ok(bytes) => Ok(bytes),
        Err(postcard::Error::SerializeBufferFull) => {
            let size = postcard::to_allocvec(envelope)
                .map(|v| v.len())
                .unwrap_or(0);
            Err(SendError::TooLarge(size))
        }
        Err(_) => Err(SendError::Encode),
    }
}

/// Deserialize an envelope from a received datagram. Returns [`None`] for
/// malformed datagrams.
pub(super) fn deserialize_envelope(bytes: &[u8]) -> Option<Envelope> {
    postcard::from_bytes(bytes).ok()
}
