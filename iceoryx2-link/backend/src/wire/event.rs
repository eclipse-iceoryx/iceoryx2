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

use iceoryx2::port::event_id::EventId;

/// The size of an event id on the wire.
pub const SIZE: usize = 8;

/// An event id as eight little-endian bytes.
pub fn encode(id: EventId) -> [u8; SIZE] {
    (id.as_value() as u64).to_le_bytes()
}

/// The event id `bytes` encode.
pub fn decode(bytes: &[u8]) -> Result<EventId, NotAnEventId> {
    let bytes: [u8; SIZE] = bytes.try_into().map_err(|_| NotAnEventId)?;
    let value = usize::try_from(u64::from_le_bytes(bytes)).map_err(|_| NotAnEventId)?;
    Ok(EventId::new(value))
}

/// The bytes are not an event id this side can hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotAnEventId;

impl core::fmt::Display for NotAnEventId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "NotAnEventId")
    }
}

impl core::error::Error for NotAnEventId {}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn an_event_id_round_trips() {
        const EVENT_ID: usize = 4711;

        let id = EventId::new(EVENT_ID);
        assert_that!(decode(&encode(id)), eq Ok(id));
    }

    #[test]
    fn bytes_of_another_length_are_no_event_id() {
        assert_that!(decode(&[0; SIZE - 1]), eq Err(NotAnEventId));
        assert_that!(decode(&[0; SIZE + 1]), eq Err(NotAnEventId));
    }
}
