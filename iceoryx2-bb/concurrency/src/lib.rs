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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod atomic;
pub mod cell;
pub mod spin_lock;

#[cfg(not(feature = "std"))]
pub use iceoryx2_pal_concurrency_sync::once::Once;
#[cfg(feature = "std")]
pub use std::sync::Once;
