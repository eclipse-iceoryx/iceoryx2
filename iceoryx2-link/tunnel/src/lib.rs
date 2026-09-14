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

//! The tunnel, the backend to another `iceoryx2` system over a carrier.
//!
//! [`Tunnel`] runs over a `Carrier`, the mechanism reaching the peers on
//! the opposing side. It is a backend a `Link` runs, the carrier crate
//! holds what a mechanism implements to be one.
//!
//! ```ignore
//! let tunnel = Tunnel::new(carrier, &config);
//! let mut link = Link::new(node, tunnel);
//! ```
//!
//! * Local services are exported, announced to the peers under their
//!   description and relayed as the bytes they are, the opposing side is
//!   another `iceoryx2` instance.
//! * A service the peers offer is mirrored with the local system's
//!   default settings from `config`, unless the peers offer it under
//!   differing types, then it is refused as disputed.
//! * Publish-subscribe and event services are relayed, each over a
//!   channel of the carrier keyed by the service's descriptor.
//! * A tunnel over a carrier that is `Reactive` is reactive itself.

#![no_std]

extern crate alloc;

mod offer_id;
pub mod relay;
pub mod resolver;
#[cfg(test)]
mod testing;
mod tunnel;

pub use offer_id::OfferId;
pub use tunnel::Tunnel;
