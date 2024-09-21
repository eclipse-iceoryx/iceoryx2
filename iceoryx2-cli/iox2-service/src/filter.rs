// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use crate::cli::OutputFilter;
use iceoryx2::service::ipc::Service;
use iceoryx2::service::ServiceDetails;
use iceoryx2_cli::filter::Filter;

impl Filter<ServiceDetails<Service>> for OutputFilter {
    fn matches(&self, service: &ServiceDetails<Service>) -> bool {
        self.pattern.matches(service)
    }
}
