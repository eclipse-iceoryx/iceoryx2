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

//! The contract a middleware implements to be bridged by a gateway.
//!
//! A gateway connects `iceoryx2` to a middleware through the
//! middleware's own endpoints. To provide one, implement:
//!
//! * [`Adapter`] for the middleware. It lists the endpoints that belong to
//!   the middleware's applications as [`EndpointDescription`]s, and opens
//!   the gateway's own endpoints for a description, creating the endpoint
//!   on the middleware if it does not exist yet.
//! * [`PublishSubscribeEndpoints`] and [`EventEndpoints`], the gateway's
//!   own endpoints on the middleware for each pattern it has, whatever
//!   the middleware offers for it. A pattern it does not have is
//!   [`UnsupportedEndpoints`], and opening it fails.
//! * [`Mapping`] decides, from the name and the settings, which endpoint
//!   represents a local service on the middleware and which local
//!   service represents an endpoint. It may refuse either with a reason
//!   of its own.
//! * [`Translator`] decides which local types correspond to the
//!   middleware's types, and whether the header and the payload of a
//!   sample need converting on the way, and how.
//!
//! Every adapter guarantees two things.
//!
//! * A gateway's own endpoints are never listed, so two gateways never
//!   mirror each other's relays.
//! * A message published reaches every other endpoint under the same
//!   description, and never comes back to its publisher.
//!
//! An adapter that can detect endpoints appearing or leaving or an arriving
//! message also implements [`iceoryx2_link_backend::Reactive`] and
//! signals the wake on detection.
//!
//! ```rust,ignore
//! impl Adapter for MyAdapter {
//!     type ListError = MyError;
//!     type OpenError = MyError;
//!     type EndpointSettings = MyEndpointSettings;
//!     type EndpointTypes = MyEndpointTypes;
//!     type PublishSubscribeEndpoints = MyEndpoints;
//!     type EventEndpoints = MyEventEndpoints;
//!
//!     fn generation(&self) -> Generation {
//!         // At(counter) if changes to the endpoints are counted, else
//!         // the default, Untracked.
//!     }
//!
//!     fn endpoints(&self, callback: &mut dyn FnMut(&EndpointDescription<..>)) -> Result<(), MyError> {
//!         // Call back once per description with remote endpoints.
//!     }
//!
//!     fn publish_subscribe(&mut self, description: &EndpointDescription<..>) -> Result<MyEndpoints, MyError> {
//!         // Open the gateway's endpoints for publish-subscribe on it.
//!     }
//!
//!     fn event(&mut self, description: &EndpointDescription<..>) -> Result<MyEventEndpoints, MyError> {
//!         // Open the gateway's endpoints for events on it.
//!     }
//! }
//!
//! impl PublishSubscribeEndpoints for MyEndpoints {
//!     type Failure = MyError;
//!
//!     fn publish(&mut self, header: &[u8], payload: &[u8]) -> Result<(), MyError> {
//!         // Publish the message to every other endpoint, the header
//!         // form and the wire form laid out as the middleware has them.
//!     }
//!
//!     fn take<D: Destination>(&mut self, into: &mut D) -> Result<bool, TakeError<MyError>> {
//!         // Write the pending message's payload in its wire form and its
//!         // header in the middleware's header form. Ask `into` for each
//!         // region by its length before writing it. Return true, or false
//!         // if nothing is pending. If `into` refuses a length, drop the
//!         // message and return the refusal.
//!     }
//! }
//!
//! impl EventEndpoints for MyEventEndpoints {
//!     type Failure = MyError;
//!
//!     fn notify(&mut self, id: EventId) -> Result<(), MyError> {
//!         // Notify every other endpoint with the id.
//!     }
//!
//!     fn take(&mut self) -> Result<Option<EventId>, MyError> {
//!         // The id of the pending notification, if any.
//!     }
//! }
//! ```

#![no_std]

extern crate alloc;

mod adapter;
mod destination;
mod endpoints;
pub mod mapping;
mod region;
pub mod translator;

pub use adapter::Adapter;
pub use destination::Destination;
pub use endpoints::{
    EndpointDescription, EventEndpoints, PublishSubscribeEndpoints, TakeError, UnsupportedEndpoints,
};
pub use mapping::Mapping;
pub use region::{Region, ResizeError};
pub use translator::{
    HeaderTranscoder, NoTranscoder, Passthrough, PayloadTranscoder, PublishSubscribeTranslation,
    SampleTranscoder, SampleTranscoders, SampleTranscodings, TranscodeError, Transcoding,
    Translator,
};
