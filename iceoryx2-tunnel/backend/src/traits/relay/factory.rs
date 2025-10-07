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

use core::fmt::Debug;
use iceoryx2::service::{static_config::StaticConfig, Service};

use crate::traits::EventRelay;
use crate::traits::PublishSubscribeRelay;

pub trait RelayBuilder {
    type CreationError: Debug;
    type Relay;

    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

pub trait RelayFactory<S: Service> {
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;
    type EventRelay: EventRelay<S>;

    type PublishSubscribeBuilder<'config>: RelayBuilder<Relay = Self::PublishSubscribeRelay>
        + Debug
        + 'config
    where
        Self: 'config;

    type EventBuilder<'config>: RelayBuilder<Relay = Self::EventRelay> + Debug + 'config
    where
        Self: 'config;

    fn publish_subscribe<'config>(
        &self,
        static_config: &'config StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'config>
    where
        Self: 'config;

    fn event<'config>(&self, static_config: &'config StaticConfig) -> Self::EventBuilder<'config>
    where
        Self: 'config;
}
