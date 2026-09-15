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

/// The event id `bytes` encode, if they are one this side can hold.
pub fn decode(bytes: &[u8]) -> Option<EventId> {
    let bytes: [u8; SIZE] = bytes.try_into().ok()?;
    let value = usize::try_from(u64::from_le_bytes(bytes)).ok()?;
    Some(EventId::new(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn an_event_id_round_trips() {
        const EVENT_ID: usize = 4711;

        let id = EventId::new(EVENT_ID);
        assert_that!(decode(&encode(id)), eq Some(id));
    }

    #[test]
    fn bytes_of_another_length_are_no_event_id() {
        assert_that!(decode(&[0; SIZE - 1]), eq None);
        assert_that!(decode(&[0; SIZE + 1]), eq None);
    }
}
