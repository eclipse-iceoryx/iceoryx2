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

//! The contract between a link and its backend, and the types crossing it.
//!
//! A link extends the local `iceoryx2` system across a boundary. It
//! knows the local system, which services are offered and which are
//! bridged, and nothing beyond. A backend knows the opposing side. One
//! kind ships, a tunnel to another `iceoryx2` system over a carrier.
//! Further kinds can be provided by implementing the traits of this
//! crate.
//!
//! A backend does four things.
//!
//! * Lists what the opposing side offers, a remote description under a
//!   remote id each, and reports a [`Generation`] that moves when the
//!   listing changed.
//! * Provides a [`resolver::Resolver`], which decides service by service
//!   what is bridged, from the local description and the remote
//!   descriptions of the service, and names the refusal when nothing is.
//! * Announces what this side offers.
//! * Provides a [`relay::RelayFactory`], which builds a relay per bridged
//!   service from its description and what it is opened as on the
//!   opposing side.
//!
//! A backend keeps track of the opposing side only. It holds no state
//! about the local system, each call receives what it needs of it, such as
//! the local description to resolve or the service to announce. A backend
//! that can detect a change on the opposing side also implements
//! [`Reactive`].
//!
//! ```rust,ignore
//! impl<S: Service> Backend<S> for MyBackend {
//!     type ListError = MyError;
//!     type AnnouncementError = MyError;
//!     type RemoteId = MyId;
//!     type RemoteDescription = MyDescription;
//!     type Refusal = MyRefusal;
//!     type Resolver<'a> = &'a MyResolver where Self: 'a;
//!     type PublishSubscribeRelay = MyPublishSubscribeRelay;
//!     type EventRelay = MyEventRelay;
//!     type RelayFactory<'a> = MyRelayFactory<'a> where Self: 'a;
//!
//!     fn generation(&self) -> Generation {
//!         // At(counter) if changes on the opposing side are counted,
//!         // otherwise Untracked.
//!     }
//!
//!     fn list(&self, on_remote: &mut OnRemote<'_, S, Self>) -> Result<(), MyError> {
//!         // Call back once per remote description, with its id.
//!     }
//!
//!     fn resolver(&self) -> &MyResolver {
//!         // The resolver defining the rules of this backend.
//!     }
//!
//!     fn announce(&mut self, announcement: Announcement<'_>) -> Result<(), MyError> {
//!         // Inform the opposing side of what this side bridges.
//!     }
//!
//!     fn relay_factory(&mut self) -> MyRelayFactory<'_> {
//!         // The factory the relays of bridged services are created with.
//!     }
//! }
//! ```

#![no_std]

extern crate alloc;

/// The origin of a log line, `site` under the module it is logged from.
#[macro_export]
macro_rules! origin {
    ($site:literal) => {
        concat!(module_path!(), "::", $site)
    };
}

mod backend;
pub mod description;
pub mod diagnostic;
mod epoch;
mod generation;
mod never;
mod reactive;
pub mod relay;
pub mod resolver;
pub mod wire;

pub use backend::{Announcement, Backend, OnRemote, Refusal, RemoteDescription, RemoteId};
pub use epoch::Epoch;
pub use generation::{Generation, GenerationCounter};
pub use never::Never;
pub use reactive::{Reactive, WakeHandle, WakeService};
