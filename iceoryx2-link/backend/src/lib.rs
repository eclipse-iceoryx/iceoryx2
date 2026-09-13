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

/// The origin of a log line, `site` under the module it is logged from.
#[macro_export]
macro_rules! origin {
    ($site:literal) => {
        concat!(module_path!(), "::", $site)
    };
}

mod backend;
pub mod description;
pub mod diagnostic;
mod epoch;
mod generation;
mod never;
mod reactive;
pub mod relay;
pub mod resolver;
pub mod wire;

pub use backend::{Announcement, Backend, OnRemote, Refusal, RemoteDescription, RemoteId};
pub use epoch::Epoch;
pub use generation::{Generation, GenerationCounter};
pub use never::Never;
pub use reactive::{Reactive, WakeHandle, WakeService};
