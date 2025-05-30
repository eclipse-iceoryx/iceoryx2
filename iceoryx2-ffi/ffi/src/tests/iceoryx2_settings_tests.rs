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

use crate::tests::*;
use iceoryx2::service::{
    attribute::{AttributeKey, AttributeValue},
    static_config::message_type_details::TypeNameString,
};

#[test]
fn ffi_settings_are_equal_to_iceoryx2_settings() {
    assert_that!(IOX2_ATTRIBUTE_KEY_LENGTH, eq AttributeKey::max_len());
    assert_that!(IOX2_ATTRIBUTE_VALUE_LENGTH, eq AttributeValue::max_len());
    assert_that!(IOX2_MAX_ATTRIBUTES_PER_SERVICE, eq AttributeSet::capacity());
    assert_that!(IOX2_SERVICE_NAME_LENGTH, eq ServiceName::max_len());
    assert_that!(IOX2_SERVICE_ID_LENGTH, eq iceoryx2::service::service_id::ServiceId::max_number_of_characters());
    assert_that!(IOX2_IS_IPC_LISTENER_FD_BASED, eq <<<ipc::Service as iceoryx2::service::Service>::Event as iceoryx2_cal::event::Event>::Listener as iceoryx2_cal::event::Listener>::IS_FILE_DESCRIPTOR_BASED);
    assert_that!(IOX2_IS_LOCAL_LISTENER_FD_BASED, eq <<<local::Service as iceoryx2::service::Service>::Event as iceoryx2_cal::event::Event>::Listener as iceoryx2_cal::event::Listener>::IS_FILE_DESCRIPTOR_BASED);
    assert_that!(IOX2_TYPE_NAME_LENGTH, eq TypeNameString::capacity());
    assert_that!(IOX2_NODE_NAME_LENGTH, eq NodeName::max_len());
}
