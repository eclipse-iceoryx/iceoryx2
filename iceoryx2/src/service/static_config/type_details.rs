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

use std::alloc::Layout;

use iceoryx2_bb_elementary::math::align;
use serde::{Deserialize, Serialize};

/// Defines if the type is a slice with a runtime-size ([`TypeVariant::Dynamic`])
/// or if its a type that satisfies [`Sized`] ([`TypeVariant::FixedSize`]).
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TypeVariant {
    #[default]
    FixedSize,
    Dynamic,
}

/// Contains all type information to the header and payload type.
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TypeDetails {
    pub variant: TypeVariant,
    pub header_type_name: String,
    pub header_size: usize,
    pub header_alignment: usize,
    pub payload_type_name: String,
    pub payload_size: usize,
    pub payload_alignment: usize,
}

impl TypeDetails {
    pub(crate) fn from<PayloadType, Header>(variant: TypeVariant) -> Self {
        Self {
            variant,
            header_type_name: core::any::type_name::<Header>().to_string(),
            header_size: core::mem::size_of::<Header>(),
            header_alignment: core::mem::align_of::<Header>(),
            payload_type_name: core::any::type_name::<PayloadType>().to_string(),
            payload_size: core::mem::size_of::<PayloadType>(),
            payload_alignment: core::mem::align_of::<PayloadType>(),
        }
    }

    pub(crate) fn payload_ptr_from_header(&self, header: *const u8) -> *const u8 {
        let header = header as usize;
        let payload_start = align(header + self.header_size, self.payload_alignment);
        payload_start as *const u8
    }

    pub(crate) fn sample_layout(&self, number_of_elements: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                align(
                    self.header_size
                        + self.payload_size * number_of_elements
                        + self.payload_alignment
                        - 1,
                    self.header_alignment,
                ),
                self.header_alignment,
            )
        }
    }

    pub(crate) fn payload_layout(&self, number_of_elements: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                self.payload_size * number_of_elements,
                self.payload_alignment,
            )
        }
    }

    pub(crate) fn is_compatible_to(&self, rhs: &Self) -> bool {
        self.variant == rhs.variant
            && self.header_type_name == rhs.header_type_name
            && self.header_size == rhs.header_size
            && self.header_alignment == rhs.header_alignment
            && self.payload_type_name == rhs.payload_type_name
            && self.payload_size == rhs.payload_size
            && self.payload_alignment <= rhs.payload_alignment
    }
}
