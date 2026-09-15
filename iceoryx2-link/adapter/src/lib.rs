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

mod destination;
mod endpoints;
pub mod mapping;
mod region;
pub mod translator;

pub use destination::Destination;
pub use endpoints::{
    EndpointDescription, EventEndpoints, PublishSubscribeEndpoints, TakeError, UnsupportedEndpoints,
};
pub use mapping::Mapping;
pub use region::{Region, ResizeError};
pub use translator::{
    HeaderTranscoder, NoTranscoder, Passthrough, PayloadTranscoder, PublishSubscribeTranslation,
    SampleTranscoder, SampleTranscoders, SampleTranscodings, TranscodeError, Transcoding,
    Translator,
};
