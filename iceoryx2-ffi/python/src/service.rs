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
    config::Config, error::ServiceDetailsError, messaging_pattern::MessagingPattern,
    service_name::ServiceName, service_type::ServiceType,
};

#[pyclass]
/// Builder to create or open `Service`s
pub struct Service(());

#[pymethods]
impl Service {
    #[staticmethod]
    pub fn does_exist(
        service_name: &ServiceName,
        config: &Config,
        messaging_pattern: MessagingPattern,
        service_type: ServiceType,
    ) -> PyResult<bool> {
        use iceoryx2::service::Service;
        match service_type {
            ServiceType::Ipc => Ok(crate::IpcService::does_exist(
                &service_name.0,
                &*config.0.lock(),
                messaging_pattern.clone().into(),
            )
            .map_err(|e| ServiceDetailsError::new_err(format!("{e:?}")))?),
            ServiceType::Local => Ok(crate::LocalService::does_exist(
                &service_name.0,
                &*config.0.lock(),
                messaging_pattern.clone().into(),
            )
            .map_err(|e| ServiceDetailsError::new_err(format!("{e:?}")))?),
        }
    }

    // TODO: details
    // TODO: list
    // TODO: ServiceDetails type
}
