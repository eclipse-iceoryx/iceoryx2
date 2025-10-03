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

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_service_name_length_e {
    VALUE = iceoryx2::constants::MAX_SERVICE_NAME_LENGTH as _,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_attributes_e {
    VALUE = iceoryx2::constants::MAX_ATTRIBUTES as _,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_attribute_key_length_e {
    VALUE = iceoryx2::constants::MAX_ATTRIBUTE_KEY_LENGTH as _,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_attribute_value_length_e {
    VALUE = iceoryx2::constants::MAX_ATTRIBUTE_VALUE_LENGTH as _,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_node_name_length_e {
    VALUE = iceoryx2::constants::MAX_NODE_NAME_LENGTH as _,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_type_name_length_e {
    VALUE = iceoryx2::constants::MAX_TYPE_NAME_LENGTH as _,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum iox2_constants_max_blackboard_key_alignment_e {
    VALUE = iceoryx2::constants::MAX_BLACKBOARD_KEY_ALIGNMENT as _,
}
