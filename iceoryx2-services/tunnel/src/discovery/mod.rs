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

pub(crate) mod state;
pub(crate) mod subscriber;
pub(crate) mod tracker;

use iceoryx2::service::Service;

use crate::discovery::subscriber::DiscoverySubscriber;
use crate::discovery::tracker::DiscoveryTracker;

/// The active local-discovery mechanism. Exactly one is in use for a given
/// [`Tunnel`](crate::Tunnel): a subscriber to a discovery service, or a
/// poll-based tracker over the local service registry.
#[derive(Debug)]
pub(crate) enum LocalDiscoveryStrategy<S: Service> {
    Subscriber(DiscoverySubscriber<S>),
    Tracker(DiscoveryTracker<S>),
}
