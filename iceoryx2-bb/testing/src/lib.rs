// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! Testing utilities and helpers.
//!
//! These utilities may leverage components in the building blocks layer of
//! the architecture.
//!
//! Components from [`iceoryx2_pal_testing`] are re-exported for convenience
//! of use in upper layers.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

extern crate alloc;

pub use iceoryx2_pal_testing::*;
pub mod instantiate_conformance_tests_macro;
pub mod lifetime_tracker;
