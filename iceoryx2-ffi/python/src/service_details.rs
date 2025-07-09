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

use crate::{
    attribute_set::AttributeSet, messaging_pattern::MessagingPattern, node_state::NodeState,
    service_id::ServiceId, service_name::ServiceName,
};

pub(crate) enum ServiceDetailsType {
    Ipc(iceoryx2::service::ServiceDetails<crate::IpcService>),
    Local(iceoryx2::service::ServiceDetails<crate::LocalService>),
}

#[pyclass]
/// Builder to create or open `Service`s
pub struct ServiceDetails(pub(crate) ServiceDetailsType);

#[pymethods]
impl ServiceDetails {
    pub fn nodes(&self) -> Vec<NodeState> {
        todo!()
    }

    pub fn attributes(&self) -> AttributeSet {
        todo!()
    }

    pub fn service_id(&self) -> ServiceId {
        todo!()
    }

    pub fn name(&self) -> ServiceName {
        todo!()
    }

    pub fn messaging_pattern(&self) -> MessagingPattern {
        todo!()
    }
}
