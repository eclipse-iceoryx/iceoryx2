// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::sync::atomic::Ordering;

use iceoryx2_cal::zero_copy_connection::{ChannelId, ZeroCopyPortDetails};

pub(crate) const INVALID_CHANNEL_STATE: u64 = u64::MAX;
const DISCONNECT_HINT_BIT: u64 = 1u64 << 63;

pub(crate) trait ChannelManagement: ZeroCopyPortDetails {
    fn set_channel_state(&self, channel_id: ChannelId, state: u64) -> bool {
        self.channel_state(channel_id)
            .compare_exchange(
                INVALID_CHANNEL_STATE,
                state,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    fn set_disconnect_hint(&self, channel_id: ChannelId, expected_state: u64) {
        let disconnect_hint_state = expected_state | DISCONNECT_HINT_BIT;

        let _ = self.channel_state(channel_id).compare_exchange(
            expected_state,
            disconnect_hint_state,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    fn has_disconnect_hint(&self, channel_id: ChannelId, expected_state: u64) -> bool {
        let disconnect_hint_state = expected_state | DISCONNECT_HINT_BIT;
        disconnect_hint_state == self.channel_state(channel_id).load(Ordering::Relaxed)
    }

    fn has_channel_state(&self, channel_id: ChannelId, expected_state: u64) -> bool {
        let state = self.channel_state(channel_id).load(Ordering::Relaxed);
        let state_without_disconnect_hint_bit = state & !(DISCONNECT_HINT_BIT);
        expected_state == state_without_disconnect_hint_bit
    }

    fn invalidate_channel_state(&self, channel_id: ChannelId, expected_state: u64) {
        match self.channel_state(channel_id).compare_exchange(
            expected_state,
            INVALID_CHANNEL_STATE,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => (),
            Err(v) => {
                let graceful_disconnect_state = expected_state | DISCONNECT_HINT_BIT;
                if v == graceful_disconnect_state {
                    let _ = self.channel_state(channel_id).compare_exchange(
                        graceful_disconnect_state,
                        INVALID_CHANNEL_STATE,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    );
                }
            }
        }
    }
}

impl<T: ZeroCopyPortDetails> ChannelManagement for T {}
