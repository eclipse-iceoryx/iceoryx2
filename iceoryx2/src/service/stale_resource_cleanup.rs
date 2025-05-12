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

use iceoryx2_bb_log::fail;
use iceoryx2_cal::event::NamedConceptMgmt;
use iceoryx2_cal::named_concept::NamedConceptRemoveError;

use crate::config;
use crate::service;
use crate::service::config_scheme::data_segment_config;
use crate::service::naming_scheme::data_segment_name;

pub(crate) unsafe fn remove_data_segment_of_port<Service: service::Service>(
    port_id: u128,
    config: &config::Config,
) -> Result<(), NamedConceptRemoveError> {
    let origin = format!(
        "remove_data_segment_of_client::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );

    fail!(from origin, when <Service::SharedMemory as NamedConceptMgmt>::remove_cfg(
            &data_segment_name(port_id),
            &data_segment_config::<Service>(config),
        ), "Unable to remove the ports ({port_id}) data segment."
    );

    Ok(())
}
