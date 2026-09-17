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

//! A carrier implementation over Zenoh, for tunnels between `iceoryx2`
//! systems on different hosts.
//!
//! Each [`ZenohCarrier`] holds one zenoh session, through which the tunnels
//! discover each other. With a default zenoh configuration, every tunnel on
//! the same network sees the services the others offer and propagates bytes
//! between them.
//!
//! A custom zenoh configuration can be provided to tune which sessions reach
//! each other and the transport they use, such as vsock between a virtual
//! machine and its host on Linux, or TLS in place of plain TCP.
//!
//! * An offer is implemented with a zenoh liveliness token. The offer remains
//!   active as long as the session is alive and the service exists in the
//!   local `iceoryx2` system.
//! * Service descriptions are shared between zenoh sessions via queryables.
//!   Zenoh sessions query other sessions for the descriptions of the
//!   services offered on their host.
//! * Samples and notifications of a service cross as bytes on zenoh keys that
//!   carry a fingerprint of the service's description, so a service only
//!   receives bytes consistent with its types. Bytes are exchanged between
//!   hosts reliably.
//! * The [`ZenohCarrier`] implements [`iceoryx2_link_backend::Reactive`] and
//!   signals the tunnel to wake up whenever an offer changes or data arrives.

mod carrier;
mod channel;
mod fingerprint;
mod inbox;
mod keys;
mod offers;

pub use carrier::{AnnouncementError, CreationError, ZenohCarrier};
pub use channel::{ChannelError, Error as ChannelSendError, ZenohChannel};
pub use keys::channels_of;
