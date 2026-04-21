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

#![no_std]

//! Backend traits and types for tunneling `iceoryx2` services across hosts.
//!
//! This crate provides the trait abstractions necessary to implement custom
//! communication backends that tunnel `iceoryx2` [`Service`](iceoryx2::service::Service)s
//! between different hosts, networks, or domains. Implementers can create backends
//! using various protocols (TCP, UDP, custom transports) while maintaining a
//! consistent interface for the `iceoryx2` tunnel infrastructure.
//!
//! # Overview
//!
//! The crate defines a hierarchy of traits that together enable complete
//! tunneling functionality:
//!
//! - [`Backend`](traits::Backend): Top-level trait combining discovery and relay factory capabilities
//! - [`Discovery`](traits::Discovery): [`Service`](iceoryx2::service::Service) announcement and discovery across the backend
//! - [`RelayFactory`](traits::RelayFactory): Factory for creating relay instances
//! - [`RelayBuilder`](traits::RelayBuilder): Builder pattern for configuring relays
//! - [`PublishSubscribeRelay`](traits::PublishSubscribeRelay): Bidirectional pub-sub data tunneling
//! - [`EventRelay`](traits::EventRelay): Bidirectional event notification tunneling
//!
//! # Architecture
//!
//! A tunnel backend implementation consists of two main components:
//!
//! 1. **Discovery**: Announces local [`Service`](iceoryx2::service::Service)s to remote hosts and discovers
//!    [`Service`](iceoryx2::service::Service)s available on remote hosts, making them accessible as if they
//!    were local.
//!
//! 2. **Relays**: Handle the actual data transmission for [`Service`](iceoryx2::service::Service)s. Each service
//!    pattern (event, publish-subscribe) has its own relay type that manages
//!    bidirectional communication over the backend's transport mechanism.
//!
//! # Usage
//!
//! This crate is intended for developers implementing custom tunnel backends.
//! End users typically interact with concrete backend implementations rather
//! than using this crate directly.
//!
//! # Features
//!
//! This crate is `no_std` compatible and requires only the `alloc` crate.

extern crate alloc;

pub mod traits;
pub mod types;
