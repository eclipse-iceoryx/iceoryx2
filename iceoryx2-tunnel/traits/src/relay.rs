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
use std::mem::MaybeUninit;

use iceoryx2::sample_mut::SampleMut;
use iceoryx2::sample_mut_uninit::SampleMutUninit;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::{
    sample::Sample,
    service::{static_config::StaticConfig, Service},
};

// TODO: Rename to Ports?
pub trait PublishSubscribeRelay<S: Service> {
    type PropagationError: Debug;
    type IngestionError: Debug;

    fn propagate(
        &self,
        sample: Sample<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ) -> Result<(), Self::PropagationError>;

    fn ingest<F>(
        &self,
        loan: &mut F,
    ) -> Result<Option<SampleMut<S, [CustomPayloadMarker], CustomHeaderMarker>>, Self::IngestionError>
    where
        F: FnMut(
            usize,
        )
            -> SampleMutUninit<S, [MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>;
}

pub trait EventRelay {
    type PropagationError: Debug;
    type IngestionError: Debug;

    fn propagate(&self) -> Result<(), Self::PropagationError>;
    fn ingest(&self) -> Result<(), Self::IngestionError>;
}

pub trait RelayBuilder {
    type CreationError: Debug;
    type Relay;

    fn create(self) -> Result<Self::Relay, Self::CreationError>;
}

pub trait RelayFactory<S: Service> {
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;
    type EventRelay: EventRelay;

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
