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
//! * [`SampleChannel`] to propagate the samples of one service, and
//!   [`EventChannel`] the ids of one event service, to peers that have the
//!   same channel open.
//!
//! Channels carry opaque bytes. They may be one stream per service or
//! every service multiplexed over one connection, as long as bytes come
//! out on the channel they went in on.
//!
//! Every carrier must guarantee two things:
//!
//! * Announcements from one carrier reach every peer in the order they
//!   were made.
//! * Bytes sent on a channel reach every other peer with that channel
//!   open, and no one else.
//!
//! A carrier that can detect a peer's announcement or arriving bytes
//! should also implement [`iceoryx2_link_backend::Reactive`] and signal
//! the wake on detection.
//!
//! ```rust,ignore
//! impl Carrier for MyCarrier {
//!     type AnnouncementError = MyError;
//!     type ListError = MyError;
//!     type ChannelError = MyError;
//!     type SampleChannel = MySampleChannel;
//!     type EventChannel = MyEventChannel;
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
//!     fn open_sample_channel(&mut self, descriptor: &ServiceDescriptor) -> Result<MySampleChannel, MyError> {
//!         // Join the peers sharing this descriptor's samples.
//!     }
//!
//!     fn open_event_channel(&mut self, descriptor: &ServiceDescriptor) -> Result<MyEventChannel, MyError> {
//!         // Join the peers sharing this descriptor's ids.
//!     }
//! }
//!
//! impl SampleChannel for MySampleChannel {
//!     type Error = MyError;
//!
//!     fn send(&mut self, bytes: &[&[u8]]) -> Result<(), MyError> {
//!         // Send the bytes, given as consecutive slices, to other peers who
//!         // have the channel open, in whatever form the transport uses.
//!     }
//!
//!     fn receive<L: LoanableSample>(&mut self, loanable: L) -> Result<Option<L::Sample>, SampleReceiveError<MyError>> {
//!         // Acquire a loan from the provided loanable and write bytes
//!         // received from the channel directly into it.
//!         //
//!         // If the loan is refused, drop the bytes and return the refusal.
//!         // Return None if there is nothing pending on the channel. Bytes
//!         // already held in a buffer are written with populate.
//!     }
//! }
//!
//! impl EventChannel for MyEventChannel {
//!     type Error = MyError;
//!
//!     fn send(&mut self, id: EventId) -> Result<(), MyError> {
//!         // Send wire::event::encode(id) to other peers who have the
//!         // channel open.
//!     }
//!
//!     fn receive(&mut self) -> Result<Option<EventId>, EventReceiveError<MyError>> {
//!         // Decode the pending bytes with wire::event::decode. Return None
//!         // if nothing is pending, Malformed if they are not an id.
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
pub use channel::{EventChannel, EventReceiveError, SampleChannel, SampleReceiveError, populate};
pub use offer::Offer;
pub use peer_id::PeerId;
