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

pub(crate) trait ChannelManagement: ZeroCopyPortDetails {
    fn set_channel_state(&self, channel_id: ChannelId, state: u64) -> bool {
        self.custom_channel_state(channel_id)
            .compare_exchange(
                INVALID_CHANNEL_STATE,
                state,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
    }

    fn has_invalid_channel_state(&self, channel_id: ChannelId) -> bool {
        self.custom_channel_state(channel_id)
            .load(Ordering::Relaxed)
            == INVALID_CHANNEL_STATE
    }

    fn invalidate_channel_state(&self, channel_id: ChannelId) {
        let mut current_state = self
            .custom_channel_state(channel_id)
            .load(Ordering::Relaxed);

        loop {
            if current_state == INVALID_CHANNEL_STATE {
                break;
            }

            match self.custom_channel_state(channel_id).compare_exchange(
                current_state,
                INVALID_CHANNEL_STATE,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => current_state = v,
            }
        }
    }
}

impl<T: ZeroCopyPortDetails> ChannelManagement for T {}
