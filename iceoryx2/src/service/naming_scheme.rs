// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use crate::port::port_identifiers::UniqueListenerId;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::FileName;

pub(crate) fn event_concept_name(listener_id: &UniqueListenerId) -> FileName {
    let msg = "The system does not support the required file name length for the listeners event concept name.";
    let origin = "event_concept_name()";
    fatal_panic!(from origin,
                 when FileName::new(listener_id.0.value().to_string().as_bytes()),
                 "{}", msg)
}

pub(crate) fn connection_name(sender_port_id: u128, receiver_port_id: u128) -> FileName {
    let mut file = FileName::new(sender_port_id.to_string().as_bytes()).unwrap();
    file.push(b'_').unwrap();
    file.push_bytes(receiver_port_id.to_string().as_bytes())
        .unwrap();
    file
}

pub(crate) fn extract_sender_port_id_from_connection(connection: &FileName) -> Option<u128> {
    if let Ok(name) = core::str::from_utf8(connection.as_bytes()) {
        if let Some(underscore_position) = name.find('_') {
            let sender_port_id = &name[..underscore_position];
            return sender_port_id.parse::<u128>().ok();
        }
    }

    None
}

pub(crate) fn extract_receiver_port_id_from_connection(connection: &FileName) -> Option<u128> {
    if let Ok(name) = core::str::from_utf8(connection.as_bytes()) {
        if let Some(underscore_position) = name.find('_') {
            if underscore_position + 1 < name.len() {
                let receiver_port_id = &name[underscore_position + 1..];
                return receiver_port_id.parse::<u128>().ok();
            }
        }
    }

    None
}

pub(crate) fn data_segment_name(port_id_value: u128) -> FileName {
    let msg = "The system does not support the required file name length for the data segment.";
    let origin = "data_segment_name()";

    fatal_panic!(from origin,
                 when FileName::new(port_id_value.to_string().as_bytes()),
                 "{}", msg)
}
