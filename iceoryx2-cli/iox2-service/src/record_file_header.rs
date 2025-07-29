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

use iceoryx2::{
    prelude::MessagingPattern, service::static_config::message_type_details::TypeDetail,
};

#[repr(C)]
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct RecordFileHeader {
    pub version: u64,
    pub payload_type: TypeDetail,
    pub header_type: TypeDetail,
    pub messaging_pattern: MessagingPattern,
}
