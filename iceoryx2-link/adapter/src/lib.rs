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
//! * [`PublishSubscribeEndpoints`], [`RequestResponseEndpoints`] and
//!   [`EventEndpoints`], the gateway's own endpoints on the middleware for
//!   each pattern it supports. For a pattern the middleware does not
//!   support, use [`UnsupportedEndpoints`]. It cannot be constructed, so
//!   the open returns an error and the gateway leaves the service
//!   unbridged.
//! * [`Mapping`] decides, from the name and the settings, which endpoint
//!   represents a local service on the middleware and which local
//!   service represents an endpoint. It may refuse either with a reason
//!   of its own.
//! * [`Translator`] decides which local types correspond to the
//!   middleware's types, and provides the transcoders that convert data
//!   between the two forms.
//!
//! Every adapter guarantees two things.
//!
//! * A gateway's own endpoints are never listed, so two gateways never
//!   mirror each other's relays.
//! * A message published reaches every other endpoint with the same
//!   description, and does not loop back.
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
//!     type EndpointTypes = EndpointTypes<MyMiddlewareTypes>;
//!     type PublishSubscribeEndpoints = MyEndpoints;
//!     type RequestResponseEndpoints = MyRequestResponseEndpoints;
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
//!         // Open the gateway's endpoints for publish-subscribe.
//!     }
//!
//!     fn request_response(&mut self, description: &EndpointDescription<..>) -> Result<MyRequestResponseEndpoints, MyError> {
//!         // Open the gateway's endpoints for request-response.
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
//!     fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), MyError> {
//!         // Publish the header and payload bytes in middleware form to all
//!         // remote endpoints.
//!     }
//!
//!     fn take<'a>(&mut self, destination: impl SampleDestination<'a>) -> Result<TakeOutcome<()>, MyError> {
//!         // Acquire buffers for the required header and payload size from
//!         // the provided `destination` and write their bytes in wire form
//!         // into them.
//!
//!         // Return:
//!         // * TakeOutcome::Taken(()) if taken bytes are written to the
//!         //   destination
//!         // * TakeOutcome::Declined if the locations were declined and drop
//!         //   the message
//!         // * TakeOutcome::Skipped for a message taken but skipped,
//!         //   such as one the endpoints published themselves
//!         // * TakeOutcome::Empty if nothing is pending
//!     }
//! }
//!
//! impl RequestResponseEndpoints for MyRequestResponseEndpoints {
//!     type Failure = MyError;
//!     type ServerRequestId = MyRequestId;
//!
//!     fn send_request(&mut self, id: ClientRequestId, request: SampleBytesRef<'_>) -> Result<(), MyError> {
//!         // Send the request bytes to every remote server. Remember `id`
//!         // to route the responses back to the requester.
//!     }
//!
//!     fn take_response<'a>(&mut self, destination: impl ResponseDestination<'a>) -> Result<TakeOutcome<(ClientRequestId, ResponseKind)>, MyError> {
//!         // Take a pending response, match it to the `id` remembered when
//!         // its request was sent and write the response in wire form into
//!         // the destination `destination.of(id)` provides for that request.
//!         // Return the `id` with the outcome so the gateway knows which
//!         // request the response or end belongs to.
//!
//!         // Return:
//!         // * TakeOutcome::Taken with the `id` and ResponseKind::Response
//!         //   if a response was written
//!         // * TakeOutcome::Taken with the `id` and ResponseKind::End if a
//!         //   remote server ended its stream
//!         // * TakeOutcome::Skipped if `destination.of(id)` returns None,
//!         //   the request is no longer held
//!     }
//!
//!     fn take_request<'a>(&mut self, destination: impl RequestDestination<'a>) -> Result<TakeOutcome<MyRequestId>, MyError> {
//!         // Take a pending request and write it in wire form into
//!         // `destination`. Return the middleware's identity of the
//!         // request with the outcome so the gateway can route the
//!         // responses back to it.
//!     }
//!
//!     fn send_response(&mut self, id: &MyRequestId, response: SampleBytesRef<'_>) -> Result<(), MyError> {
//!         // Send the response bytes to the remote client that sent the
//!         // request `id` identifies.
//!     }
//!
//!     fn end_response_stream(&mut self, id: MyRequestId) -> Result<(), MyError> {
//!         // Tell the remote client that sent the request `id` identifies
//!         // that no more responses follow.
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
mod endpoints;
pub mod mapping;
pub mod translator;

pub use adapter::Adapter;
pub use endpoints::{
    EndpointDescription, EndpointTypes, EventEndpoints, PublishSubscribeEndpoints,
    RequestDestination, RequestResponseEndpoints, ResponseDestination, ResponseKind,
    SampleDestination, TakeOutcome, UnsupportedEndpoints,
};
pub use iceoryx2_link_backend::relay::ReceiveOutcome;
pub use iceoryx2_link_backend::wire::request_response::ClientRequestId;
pub use iceoryx2_link_backend::wire::sample::{
    LoanError, LoanableSample, SampleBytes, SampleBytesRef, SampleBytesRefMut, SampleLengths,
    WritableSample,
};
pub use mapping::Mapping;
pub use translator::{
    LocalTypes, NoTranscoder, Passthrough, SampleShape, SampleTranscoders, Shape, TranscodeError,
    Transcoder, TranscodesSamples, Translator, Translators, UnsupportedShape,
    UnsupportedTranslator,
};
