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

//! A carrier over zenoh, tunnelling `iceoryx2` systems across hosts.
//!
//! [`ZenohCarrier`] implements the carrier contract on one zenoh session.
//! Offers are liveliness tokens, one per service and description, with a
//! queryable beside each serving the descriptor. Frames cross on a key
//! per service and description.

mod carrier;
mod channel;
mod fingerprint;
mod inbox;
mod keys;
mod offers;

pub use carrier::{AnnouncementError, CreationError, ZenohCarrier};
pub use channel::{ChannelError, Error as ChannelSendError, ZenohChannel};
pub use keys::channels_of;
