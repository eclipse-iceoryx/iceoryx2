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
    config::Config,
    error::{ServiceDetailsError, ServiceListError},
    messaging_pattern::MessagingPattern,
    service_details::{ServiceDetails, ServiceDetailsType},
    service_name::ServiceName,
    service_type::ServiceType,
};

#[pyclass]
/// Builder to create or open `Service`s
pub struct Service(());

#[pymethods]
impl Service {
    #[staticmethod]
    /// Checks if a service under a given `Config` does exist
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
                &config.0.lock(),
                messaging_pattern.clone().into(),
            )
            .map_err(|e| ServiceDetailsError::new_err(format!("{e:?}")))?),
            ServiceType::Local => Ok(crate::LocalService::does_exist(
                &service_name.0,
                &config.0.lock(),
                messaging_pattern.clone().into(),
            )
            .map_err(|e| ServiceDetailsError::new_err(format!("{e:?}")))?),
        }
    }

    #[staticmethod]
    /// Acquires the `ServiceDetails` of a `Service`.
    pub fn details(
        service_name: &ServiceName,
        config: &Config,
        messaging_pattern: MessagingPattern,
        service_type: ServiceType,
    ) -> PyResult<Option<ServiceDetails>> {
        use iceoryx2::service::Service;
        match service_type {
            ServiceType::Ipc => Ok(crate::IpcService::details(
                &service_name.0,
                &config.0.lock(),
                messaging_pattern.into(),
            )
            .map_err(|e| ServiceDetailsError::new_err(format!("{e:?}")))?
            .map(|details| ServiceDetails(ServiceDetailsType::Ipc(details)))),
            ServiceType::Local => Ok(crate::LocalService::details(
                &service_name.0,
                &config.0.lock(),
                messaging_pattern.into(),
            )
            .map_err(|e| ServiceDetailsError::new_err(format!("{e:?}")))?
            .map(|details| ServiceDetails(ServiceDetailsType::Local(details)))),
        }
    }

    #[staticmethod]
    /// Returns a list of all services created under a given `Config`.
    pub fn list(config: &Config, service_type: ServiceType) -> PyResult<Vec<ServiceDetails>> {
        use iceoryx2::service::Service;
        let mut ret_val = vec![];
        match service_type {
            ServiceType::Ipc => crate::IpcService::list(&config.0.lock(), |service| {
                ret_val.push(ServiceDetails(ServiceDetailsType::Ipc(service)));
                iceoryx2::prelude::CallbackProgression::Continue
            })
            .map_err(|e| ServiceListError::new_err(format!("{e:?}")))?,
            ServiceType::Local => crate::LocalService::list(&config.0.lock(), |service| {
                ret_val.push(ServiceDetails(ServiceDetailsType::Local(service)));
                iceoryx2::prelude::CallbackProgression::Continue
            })
            .map_err(|e| ServiceListError::new_err(format!("{e:?}")))?,
        };

        Ok(ret_val)
    }
}
