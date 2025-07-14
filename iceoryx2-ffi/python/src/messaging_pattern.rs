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

use pyo3::prelude::*;

#[pyclass(eq, eq_int)]
#[derive(PartialEq, Clone, Debug)]
pub enum MessagingPattern {
    PublishSubscribe,
    Event,
    RequestResponse,
    Blackboard,
}

#[pymethods]
impl MessagingPattern {
    pub fn __str__(&self) -> String {
        format!("{self:?}")
    }
}

impl From<iceoryx2::prelude::MessagingPattern> for MessagingPattern {
    fn from(value: iceoryx2::prelude::MessagingPattern) -> Self {
        match value {
            iceoryx2::prelude::MessagingPattern::Event => MessagingPattern::Event,
            iceoryx2::prelude::MessagingPattern::RequestResponse => {
                MessagingPattern::RequestResponse
            }
            iceoryx2::prelude::MessagingPattern::PublishSubscribe => {
                MessagingPattern::PublishSubscribe
            }
            iceoryx2::prelude::MessagingPattern::Blackboard => MessagingPattern::Blackboard,
        }
    }
}

impl From<MessagingPattern> for iceoryx2::prelude::MessagingPattern {
    fn from(value: MessagingPattern) -> Self {
        match value {
            MessagingPattern::Event => iceoryx2::prelude::MessagingPattern::Event,
            MessagingPattern::RequestResponse => {
                iceoryx2::prelude::MessagingPattern::RequestResponse
            }
            MessagingPattern::PublishSubscribe => {
                iceoryx2::prelude::MessagingPattern::PublishSubscribe
            }
            MessagingPattern::Blackboard => iceoryx2::prelude::MessagingPattern::Blackboard,
        }
    }
}
