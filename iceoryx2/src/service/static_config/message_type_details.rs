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
    /// A fixed size type like [`u64`]
    FixedSize,
    /// A dynamic sized type like a slice
    Dynamic,
}

/// Contains all type details required to connect to a [`crate::service::Service`]
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct TypeDetail {
    /// The [`TypeVariant`] of the type
    pub variant: TypeVariant,
    /// Contains the output of [`core::any::type_name()`].
    pub type_name: String,
    /// The size of the underlying type.
    pub size: usize,
    /// The alignment of the underlying type.
    pub alignment: usize,
}

impl TypeDetail {
    #[doc(hidden)]
    pub fn __internal_new<T>(variant: TypeVariant) -> Self {
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
    /// The [`TypeDetail`] of the header of a message, the first iceoryx2 internal part.
    pub header: TypeDetail,
    /// The [`TypeDetail`] of the user_header or the custom header, is located directly after the
    /// header.
    pub user_header: TypeDetail,
    /// The [`TypeDetail`] of the payload of the message, the last part.
    pub payload: TypeDetail,
}

impl MessageTypeDetails {
    pub(crate) fn from<Header, UserHeader, Payload>(variant: TypeVariant) -> Self {
        Self {
            header: TypeDetail::__internal_new::<Header>(TypeVariant::FixedSize),
            user_header: TypeDetail::__internal_new::<UserHeader>(TypeVariant::FixedSize),
            payload: TypeDetail::__internal_new::<Payload>(variant),
        }
    }

    pub(crate) fn payload_ptr_from_header(&self, header: *const u8) -> *const u8 {
        let user_header = self.user_header_ptr_from_header(header) as usize;
        let payload_start = align(user_header + self.user_header.size, self.payload.alignment);
        payload_start as *const u8
    }

    pub(crate) fn user_header_ptr_from_header(&self, header: *const u8) -> *const u8 {
        let header = header as usize;
        let user_header_start = align(header + self.header.size, self.user_header.alignment);
        user_header_start as *const u8
    }

    pub(crate) fn sample_layout(&self, number_of_elements: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                align(
                    self.header.size + self.user_header.size + self.user_header.alignment - 1
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
            && self.user_header.type_name == rhs.user_header.type_name
            && self.user_header.variant == rhs.user_header.variant
            && self.user_header.size == rhs.user_header.size
            && self.user_header.alignment <= rhs.user_header.alignment
            && self.payload.type_name == rhs.payload.type_name
            && self.payload.variant == rhs.payload.variant
            && self.payload.size == rhs.payload.size
            && self.payload.alignment <= rhs.payload.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn test_payload_ptr_from_header(){
        let details = MessageTypeDetails::from::<i32,bool,i32>(TypeVariant::Dynamic);
        struct Demo {
            header: i32,
            _user_header: bool,
            _payload: i32,
        }
        
        let demo = Demo{
            header: 123,
            _user_header: true,
            _payload:9999,
        };

        let ptr: *const u8 = &demo.header as *const _ as *const u8;
        let payload_ptr = details.payload_ptr_from_header(ptr) as *const i32;
        let got = unsafe { *payload_ptr };
        assert_that!(got, eq 9999);
    }
}