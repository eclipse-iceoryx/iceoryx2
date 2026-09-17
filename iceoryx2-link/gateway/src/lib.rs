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

//! The gateway, the backend to another middleware over an adapter.
//!
//! [`Gateway`] runs over an `Adapter` to integrate with the middleware's
//! endpoints, with a `Mapping` to map services to endpoints, and a
//! `Translator` to translate their types and data. It is one of the
//! backend implementations of a `Link`.
//!
//! ```ignore
//! let gateway = Gateway::new(adapter, mapping, translator);
//! let mut link = Link::new(node, gateway);
//! ```
//!
//! * A local service is exported to the endpoint the mapping takes it to,
//!   with the types the translator gives it.
//! * An endpoint no local service exports to is mirrored as the service
//!   the mapping and the translator compose for it.
//! * A service or endpoint is left unbridged, with the reason named,
//!   when the mapping refuses it or leads back to another name, when two
//!   endpoints map to one service, or when the translator has no
//!   counterpart for the types.
//! * Publish-subscribe services are relayed through the translator's
//!   transcoder, event services are not bridged.
//! * A gateway over an adapter that is `Reactive` is reactive itself.
#![no_std]

extern crate alloc;

mod gateway;
pub mod relay;
pub mod resolver;
#[cfg(test)]
mod testing;

pub use gateway::Gateway;
