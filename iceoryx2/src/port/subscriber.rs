// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use super::details::publisher_connections::ConnectionFailure;

/// Defines the failure that can occur when receiving data with [`Subscriber::receive()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberReceiveError {
    ExceedsMaxBorrowedSamples,
    ConnectionFailure(ConnectionFailure),
}

impl std::fmt::Display for SubscriberReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SubscriberReceiveError {}

/// Describes the failures when a new [`Subscriber`] is created via the
/// [`crate::service::port_factory::subscriber::PortFactorySubscriber`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SubscriberCreateError {
    ExceedsMaxSupportedSubscribers,
}

impl std::fmt::Display for SubscriberCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SubscriberCreateError {}

pub(crate) mod internal {
    use std::fmt::Debug;

    pub(crate) trait SubscriberMgmt: Debug {
        fn release_sample(&self, channel_id: usize, sample: usize);
    }
}
