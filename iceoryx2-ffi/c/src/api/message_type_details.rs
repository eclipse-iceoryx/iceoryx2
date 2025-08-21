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

use core::ffi::c_char;

use iceoryx2::service::static_config::message_type_details::*;

use crate::{iox2_type_variant_e, IOX2_TYPE_NAME_LENGTH};

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_type_detail_t {
    pub variant: iox2_type_variant_e,
    pub type_name: [c_char; IOX2_TYPE_NAME_LENGTH],
    pub size: usize,
    pub alignment: usize,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct iox2_message_type_details_t {
    pub header: iox2_type_detail_t,
    pub user_header: iox2_type_detail_t,
    pub payload: iox2_type_detail_t,
}

impl From<&TypeDetail> for iox2_type_detail_t {
    fn from(value: &TypeDetail) -> Self {
        Self {
            variant: (&value.variant()).into(),
            type_name: core::array::from_fn(|n| {
                if n < value.type_name().len() {
                    value.type_name().as_bytes()[n] as _
                } else {
                    0
                }
            }),
            size: value.size(),
            alignment: value.alignment(),
        }
    }
}

impl From<&MessageTypeDetails> for iox2_message_type_details_t {
    fn from(m: &MessageTypeDetails) -> Self {
        Self {
            header: (&m.header).into(),
            user_header: (&m.user_header).into(),
            payload: (&m.payload).into(),
        }
    }
}
