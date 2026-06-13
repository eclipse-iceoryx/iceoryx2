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

use core::fmt::Debug;

/// Strategy for translating payload bytes between the wire format and the
/// iceoryx2 payload.
pub trait Translator: Default + Debug + Send + 'static {}

/// The identity [`Translator`]: payloads cross unmodified in both directions.
#[derive(Debug, Default, Clone, Copy)]
pub struct Passthrough;

impl Translator for Passthrough {}
