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

#![no_std]

extern crate alloc;

mod adapter;
mod carrier;

pub use adapter::{
    FakeAdapter, FakeEndpointDescription, FakeEndpointSettings, FakeEndpointTypes, FakeEndpoints,
    FakeMapping, FakeMiddleware, FakeSwapTranslator, SwapTranscoder, endpoint_of, swapped_bytes,
};
pub use carrier::{Error, FakeBus, FakeCarrier, FakeEventChannel, FakeSampleChannel};
