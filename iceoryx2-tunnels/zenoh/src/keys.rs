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

use iceoryx2::service::service_id::ServiceId;

pub fn data_stream(service_id: &ServiceId) -> String {
    format!("iox2/services/{}/stream", service_id.as_str())
}

pub fn service(service_id: &ServiceId) -> String {
    format!("iox2/services/{}", service_id.as_str())
}

pub fn all_services() -> String {
    "iox2/services/*".into()
}
