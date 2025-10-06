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

use core::mem::MaybeUninit;

use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;

pub type Header = CustomHeaderMarker;
pub type Payload = [CustomPayloadMarker];
pub type PayloadUninit = [MaybeUninit<CustomPayloadMarker>];

pub type Sample<S> = iceoryx2::sample::Sample<S, Payload, Header>;
pub type SampleMut<S> = iceoryx2::sample_mut::SampleMut<S, Payload, Header>;
pub type SampleMutUninit<S> =
    iceoryx2::sample_mut_uninit::SampleMutUninit<S, PayloadUninit, Header>;
pub type Publisher<S> = iceoryx2::port::publisher::Publisher<S, Payload, Header>;
pub type Subscriber<S> = iceoryx2::port::subscriber::Subscriber<S, Payload, Header>;

pub type LoanFn<'a, S> = dyn FnMut(usize) -> SampleMutUninit<S> + 'a;
