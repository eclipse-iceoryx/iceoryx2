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
    let name = core::str::from_utf8(connection.as_bytes()).ok()?;
    let (sender_port_id, _) = name.split_once('_')?;
    sender_port_id.parse::<u128>().ok()
}

pub(crate) fn extract_receiver_port_id_from_connection(connection: &FileName) -> Option<u128> {
    let name = core::str::from_utf8(connection.as_bytes()).ok()?;
    let (_, receiver_port_id) = name.split_once('_')?;
    receiver_port_id.parse::<u128>().ok()
}

pub(crate) fn data_segment_name(port_id_value: u128) -> FileName {
    let msg = "The system does not support the required file name length for the data segment.";
    let origin = "data_segment_name()";

    fatal_panic!(from origin,
                 when FileName::new(port_id_value.to_string().as_bytes()),
                 "{}", msg)
}

pub(crate) fn blackboard_name(service_id: &str) -> FileName {
    let msg = "The system does not support the required file name length for the blackboard's management segment.";
    let origin = "blackboard_name()";

    fatal_panic!(from origin, when FileName::new(service_id.as_bytes()), "{}", msg)
}
