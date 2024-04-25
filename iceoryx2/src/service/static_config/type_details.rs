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

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum TypeVariant {
    FixedSize,
    Dynamic,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TypeDetails {
    pub variant: TypeVariant,
    pub header_type_name: String,
    pub header_size: usize,
    pub header_alignment: usize,
    pub message_type_name: String,
    pub message_size: usize,
    pub message_alignment: usize,
}

impl TypeDetails {
    pub fn from<MessageType, Header>(variant: TypeVariant) -> Self {
        Self {
            variant,
            header_type_name: core::any::type_name::<Header>().to_string(),
            header_size: core::mem::size_of::<Header>(),
            header_alignment: core::mem::align_of::<Header>(),
            message_type_name: core::any::type_name::<MessageType>().to_string(),
            message_size: core::mem::size_of::<MessageType>(),
            message_alignment: core::mem::align_of::<MessageType>(),
        }
    }

    pub fn sample_layout(&self, number_of_elements: usize) -> Layout {
        let aligned_header_size = align(self.header_size, self.message_alignment);
        unsafe {
            Layout::from_size_align_unchecked(
                align(
                    aligned_header_size + self.message_size * number_of_elements,
                    self.header_alignment,
                ),
                self.header_alignment,
            )
        }
    }

    pub fn message_layout(&self, number_of_elements: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                self.message_size * number_of_elements,
                self.message_alignment,
            )
        }
    }

    pub fn is_compatible(&self, rhs: &Self) -> bool {
        self.variant == rhs.variant
            && self.header_type_name == rhs.header_type_name
            && self.header_size == rhs.header_size
            && self.header_alignment == rhs.header_alignment
            && self.message_type_name == rhs.message_type_name
            && self.message_size == rhs.message_size
            && self.message_alignment <= rhs.message_alignment
    }
}
