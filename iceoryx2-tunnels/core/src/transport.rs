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

/// Abstraction of the transport over which data in iceoryx2 is propagated.
///
/// Enables implementations to define custom initialization logic.
pub trait Transport: Sized {
    type Config: Default + Debug;
    type CreationError;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError>;
}
