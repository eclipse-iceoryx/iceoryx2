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

use crate::port::port_identifiers::{UniqueListenerId, UniquePublisherId, UniqueSubscriberId};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::FileName;

use super::static_config::StaticConfig;

pub(crate) fn event_concept_name(listener_id: &UniqueListenerId) -> FileName {
    let msg = "The system does not support the required file name length for the listeners event concept name.";
    let origin = "event_concept_name()";
    let mut file = fatal_panic!(from origin, when FileName::new(listener_id.0.pid().to_string().as_bytes()), "{}", msg);
    fatal_panic!(from origin, when file.push(b'_'), "{}", msg);
    fatal_panic!(from origin, when file.push_bytes(listener_id.0.value().to_string().as_bytes()), "{}", msg);
    file
}

pub(crate) fn dynamic_config_storage_name(static_config: &StaticConfig) -> FileName {
    FileName::new(static_config.uuid().as_bytes()).unwrap()
}

pub(crate) fn static_config_storage_name(uuid: &str) -> FileName {
    FileName::new(uuid.as_bytes()).unwrap()
}

pub(crate) fn connection_name(
    publisher_id: UniquePublisherId,
    subscriber_id: UniqueSubscriberId,
) -> FileName {
    let mut file = FileName::new(publisher_id.0.value().to_string().as_bytes()).unwrap();
    file.push(b'_').unwrap();
    file.push_bytes(subscriber_id.0.value().to_string().as_bytes())
        .unwrap();
    file
}

pub(crate) fn data_segment_name(publisher_id: UniquePublisherId) -> FileName {
    let msg = "The system does not support the required file name length for the publishers data segment.";
    let origin = "data_segment_name()";

    let mut file = fatal_panic!(from origin, when FileName::new(publisher_id.0.pid().to_string().as_bytes()), "{}", msg);
    fatal_panic!(from origin, when file.push(b'_'), "{}", msg);
    fatal_panic!(from origin, when file.push_bytes(publisher_id.0.value().to_string().as_bytes()), "{}", msg);
    file
}
