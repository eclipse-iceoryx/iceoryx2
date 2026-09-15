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

use iceoryx2_link_adapter::{Passthrough, Translator};
use iceoryx2_link_testing::{FakeEndpointTypes, FakeSwapTranslator};

/// The payload type every suite speaks through the fixtures.
pub(crate) type Payload = u64;

/// The form the fake middleware carries data in, as the far application
/// writes and reads it, paired with the translator that serves it.
pub(crate) trait WireForm: Translator<EndpointTypes = FakeEndpointTypes> + Default {}

impl WireForm for Passthrough {}

impl WireForm for FakeSwapTranslator {}
