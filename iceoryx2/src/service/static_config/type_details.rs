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

/// Contains all type details required to connect to a [`crate::service::Service`]
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TypeDetail {
    pub variant: TypeVariant,
    pub type_name: String,
    pub size: usize,
    pub alignment: usize,
}

impl TypeDetail {
    fn new<T>(variant: TypeVariant) -> Self {
        Self {
            variant,
            type_name: core::any::type_name::<T>().to_string(),
            size: core::mem::size_of::<T>(),
            alignment: core::mem::align_of::<T>(),
        }
    }
}

/// Contains all type information to the header and payload type.
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct MessageTypeDetails {
    pub header: TypeDetail,
    pub payload: TypeDetail,
}

impl MessageTypeDetails {
    pub(crate) fn from<Payload, Header>(variant: TypeVariant) -> Self {
        Self {
            header: TypeDetail::new::<Header>(TypeVariant::FixedSize),
            payload: TypeDetail::new::<Payload>(variant),
        }
    }

    pub(crate) fn payload_ptr_from_header(&self, header: *const u8) -> *const u8 {
        let header = header as usize;
        let payload_start = align(header + self.header.size, self.payload.alignment);
        payload_start as *const u8
    }

    pub(crate) fn sample_layout(&self, number_of_elements: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                align(
                    self.header.size
                        + self.payload.size * number_of_elements
                        + self.payload.alignment
                        - 1,
                    self.header.alignment,
                ),
                self.header.alignment,
            )
        }
    }

    pub(crate) fn payload_layout(&self, number_of_elements: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                self.payload.size * number_of_elements,
                self.payload.alignment,
            )
        }
    }

    pub(crate) fn is_compatible_to(&self, rhs: &Self) -> bool {
        self.header == rhs.header
            && self.payload.type_name == rhs.payload.type_name
            && self.payload.variant == rhs.payload.variant
            && self.payload.size == rhs.payload.size
            && self.payload.alignment <= rhs.payload.alignment
    }
}
