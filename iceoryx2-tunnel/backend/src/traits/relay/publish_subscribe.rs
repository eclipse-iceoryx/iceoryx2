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
use iceoryx2::{sample::Sample, service::Service};

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
