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

//! The contract a mechanism implements to carry bytes of a tunnel.
//!
//! A tunnel connects `iceoryx2` systems over a mechanism such as a
//! network protocol or a hypervisor channel. To provide one, implement:
//!
//! * [`Carrier`] for the mechanism. It announces what this side offers,
//!   an [`Announcement`] at a time, lists what the peers offer, an
//!   [`Offer`] per service per peer, and opens a channel per service.
//! * [`Channel`] to propagate the bytes of a single service to peers
//!   that have the same channel open.
//!
//! Frames are opaque bytes. Channels may be one stream per service or
//! every service multiplexed over one connection, as long as a frame
//! comes out on the channel it went in on.
//!
//! Every carrier must guarantee two things:
//!
//! * Announcements from one carrier reach every peer in the order they
//!   were made.
//! * A frame sent on a channel reaches every other peer with that
//!   channel open, and no one else.
//!
//! A carrier that can detect a peer's announcement or an arriving frame
//! should also implement [`iceoryx2_link_backend::Reactive`] and signal
//! the wake on detection.
//!
//! ```rust,ignore
//! impl Carrier for MyCarrier {
//!     type AnnouncementError = MyError;
//!     type ListError = MyError;
//!     type ChannelError = MyError;
//!     type Channel = MyChannel;
//!
//!     fn announce(&mut self, announcement: Announcement) -> Result<(), MyError> {
//!         // Publish the offer or the withdrawal to the peers.
//!     }
//!
//!     fn generation(&self) -> Generation {
//!         // At(counter) if changes to the peers' offers are counted,
//!         // else the default, Untracked.
//!     }
//!
//!     fn offers(&self, callback: &mut dyn FnMut(Offer)) -> Result<(), MyError> {
//!         // Call back once per offer of every peer but this one.
//!     }
//!
//!     fn open_channel(&mut self, descriptor: &ServiceDescriptor) -> Result<MyChannel, MyError> {
//!         // Join the peers sharing this descriptor's byte stream.
//!     }
//! }
//!
//! impl Channel for MyChannel {
//!     type Error = MyError;
//!
//!     fn send(&mut self, header: &[u8], payload: &[u8]) -> Result<(), MyError> {
//!         // Send the provided header and the payload to other peers who have
//!         // the channel open.
//!     }
//!
//!     fn receive<L: LoanableSample>(&mut self, loanable: L) -> Result<Option<L::Sample>, ReceiveError<MyError>> {
//!         // Acquire a loan from the provided `loanable` and write bytes
//!         // received from the channel directly into it.
//!         //
//!         // If the loan is refused, drop the frame and return the refusal.
//!         // Return None if there is nothing pending on the channel.
//!     }
//! }
//! ```

#![no_std]

extern crate alloc;

mod announcement;
mod carrier;
mod channel;
mod offer;
mod peer_id;

pub use announcement::Announcement;
pub use carrier::Carrier;
pub use channel::{Channel, ReceiveError, header_size, populate};
pub use offer::Offer;
pub use peer_id::PeerId;
