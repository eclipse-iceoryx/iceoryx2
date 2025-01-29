// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use core::fmt::Debug;

use tiny_fn::tiny_fn;

pub(crate) mod details;

/// Sends requests to a [`Server`](crate::port::server::Server) and receives responses.
pub mod client;
/// Defines the event id used to identify the source of an event.
pub mod event_id;
/// Receiving endpoint (port) for event based communication
pub mod listener;
/// Sending endpoint (port) for event based communication
pub mod notifier;
/// Defines port specific unique ids. Used to identify source/destination while communicating.
pub mod port_identifiers;
/// Sending endpoint (port) for publish-subscribe based communication
pub mod publisher;
/// Receives requests from a [`Client`](crate::port::client::Client) port and sends back responses.
pub mod server;
/// Receiving endpoint (port) for publish-subscribe based communication
pub mod subscriber;
/// Interface to perform cyclic updates to the ports. Required to deliver history to new
/// participants or to perform other management tasks.
pub mod update_connections;

use crate::port::port_identifiers::*;
use crate::service;

/// Defines the action a port shall take when an internal failure occurs. Can happen when the
/// system is corrupted and files are modified by non-iceoryx2 instances. Is used as return value of
/// the [`DegrationCallback`] to define a custom behavior.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DegrationAction {
    /// Ignore the degration completely
    Ignore,
    /// Print out a warning as soon as the degration is detected
    Warn,
    /// Returns a failure in the function the degration was detected
    Fail,
}

tiny_fn! {
    /// Defines a custom behavior whenever a port detects a degregation.
    pub struct DegrationCallback = Fn(service: &service::static_config::StaticConfig, sender_port_id: u128, receiver_port_id: u128) -> DegrationAction;
}

impl Debug for DegrationCallback<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}
