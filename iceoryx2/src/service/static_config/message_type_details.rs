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

use core::alloc::Layout;

use iceoryx2_bb_container::byte_string::FixedSizeByteString;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::math::align;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use serde::{Deserialize, Serialize};

use crate::constants::MAX_TYPE_NAME_LENGTH;

/// Defines if the type is a slice with a runtime-size ([`TypeVariant::Dynamic`])
/// or if its a type that satisfies [`Sized`] ([`TypeVariant::FixedSize`]).
#[repr(C)]
#[derive(
    Default, Debug, Clone, Copy, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize,
)]
pub enum TypeVariant {
    #[default]
    /// A type notated by [`#[repr(C)]`](https://doc.rust-lang.org/reference/type-layout.html#reprc).
    /// with a constant size known at compile time is recognized as FixedSize.
    /// The FixedSize type should satisfy the [`Sized`].
    /// For example, all primitive types are FixedSize. The self-contained structs(without pointer members
    /// or heap-usages) are FixedSize.
    FixedSize,

    /// A dynamic sized type strictly refers to the slice of an iceoryx2 compatible types.
    /// The struct with pointer members or with heap usage MUSTN't be recognized as Dynamic type.
    /// Indeed, they're the in-compatible iceoryx2 types.
    ///
    /// The underlying reason is the shared memory which we use to store the payload data.
    /// If the payload type would use the heap then the type would use
    /// process local memory that is not available to another process.
    ///
    /// The pointer requirement comes again from shared memory.
    /// It has a different pointer address offset in every process rendering any absolute pointer
    /// useless and dereferencing it would end up in a segfault.
    Dynamic,
}

/// A fixed-size string type used to store type names.
pub type TypeNameString = FixedSizeByteString<MAX_TYPE_NAME_LENGTH>;

/// Contains all type details required to connect to a [`crate::service::Service`]
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct TypeDetail {
    pub(crate) variant: TypeVariant,
    pub(crate) type_name: TypeNameString,
    pub(crate) size: usize,
    pub(crate) alignment: usize,
}

impl TypeDetail {
    /// Creates a new [`TypeDetail`] from the provided `T`. The [`TypeVariant`] defines if
    /// the type is part of a slice or directly contained.
    pub fn new<T: ZeroCopySend>(variant: TypeVariant) -> Self {
        Self {
            variant,
            type_name: unsafe {
                fatal_panic!(
                    from "TypeDetail::__internal_new::<T>()",
                    when TypeNameString::try_from(T::type_name()),
                    "Name of type T does not fit into fixed-size TypeNameString"
                )
            },
            size: core::mem::size_of::<T>(),
            alignment: core::mem::align_of::<T>(),
        }
    }

    /// The [`TypeVariant`] of the type
    pub fn variant(&self) -> TypeVariant {
        self.variant
    }

    /// Contains the name of the underlying type.
    pub fn type_name(&self) -> &TypeNameString {
        &self.type_name
    }

    /// The size of the underlying type calculated by [`core::mem::size_of`].
    pub fn size(&self) -> usize {
        self.size
    }

    /// The ABI-required minimum alignment of the underlying type calculated by [`core::mem::align_of`].
    /// It may be set by users with a larger alignment, e.g. the memory provided by allocator used by SIMD.
    pub fn alignment(&self) -> usize {
        self.alignment
    }
}

/// Contains all type information to the header and payload type.
#[derive(Default, Debug, Clone, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
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
    pub(crate) fn from<Header: ZeroCopySend, UserHeader: ZeroCopySend, Payload: ZeroCopySend>(
        payload_variant: TypeVariant,
    ) -> Self {
        Self {
            header: TypeDetail::new::<Header>(TypeVariant::FixedSize),
            user_header: TypeDetail::new::<UserHeader>(TypeVariant::FixedSize),
            payload: TypeDetail::new::<Payload>(payload_variant),
        }
    }

    pub(crate) fn payload_ptr_from_header(&self, header: *const u8) -> *const u8 {
        let user_header = self.user_header_ptr_from_header(header) as usize;
        let payload_start = align(user_header + self.user_header.size, self.payload.alignment);
        payload_start as *const u8
    }

    /// returns the pointer to the user header
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
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_testing::assert_that;

    #[cfg(target_pointer_width = "32")]
    const ALIGNMENT: usize = 4;
    #[cfg(target_pointer_width = "64")]
    const ALIGNMENT: usize = 8;

    #[test]
    fn test_from() {
        #[derive(ZeroCopySend)]
        #[repr(C)]
        struct MyPayload {
            _a: i32,
            _b: bool,
            _c: i64,
        }

        let sut = MessageTypeDetails::from::<i32, i64, MyPayload>(TypeVariant::FixedSize);
        let expected = MessageTypeDetails{
            header:  TypeDetail{
                variant: TypeVariant::FixedSize,
                type_name: "i32".try_into().unwrap(),
                size: 4,
                alignment: 4, // i32 uses 4 bytes, so its aliment is always 4 no matter x32 or x64.
            },
            user_header: TypeDetail{
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: ALIGNMENT,
            },
            payload: TypeDetail{
                variant: TypeVariant::FixedSize,
                type_name: "iceoryx2::service::static_config::message_type_details::tests::test_from::MyPayload".try_into().unwrap(),
                size: 16,
                alignment: ALIGNMENT,
            },
        };
        assert_that!(sut, eq expected);

        let sut = MessageTypeDetails::from::<i32, bool, i64>(TypeVariant::Dynamic);
        let expected = MessageTypeDetails {
            header: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i32".try_into().unwrap(),
                size: 4,
                alignment: 4,
            },
            user_header: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "bool".try_into().unwrap(),
                size: 1,
                alignment: 1,
            },
            payload: TypeDetail {
                variant: TypeVariant::Dynamic,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: ALIGNMENT,
            },
        };
        assert_that!(sut, eq expected);
    }

    #[test]
    fn test_user_header_ptr_from_header() {
        let details = MessageTypeDetails::from::<i32, bool, i64>(TypeVariant::Dynamic);
        #[repr(C)]
        struct Demo {
            header: i32,
            user_header: bool,
            _payload: i64,
        }

        let demo = Demo {
            header: 123,
            user_header: true,
            _payload: 123,
        };

        let ptr: *const u8 = &demo.header as *const _ as *const u8;
        let user_header_ptr = details.user_header_ptr_from_header(ptr);
        let sut: *const bool = user_header_ptr as *const bool;
        assert_that!(unsafe { *sut } , eq demo.user_header);

        let details = MessageTypeDetails::from::<i64, i32, i64>(TypeVariant::Dynamic);
        #[repr(C)]
        struct Demo2 {
            header: i64,
            user_header: i32,
            _payload: i64,
        }

        let demo = Demo2 {
            header: 123,
            user_header: 999,
            _payload: 123,
        };

        let ptr: *const u8 = &demo.header as *const _ as *const u8;
        let user_header_ptr = details.user_header_ptr_from_header(ptr);
        let sut: *const i32 = user_header_ptr as *const i32;
        assert_that!(unsafe { *sut } , eq demo.user_header);
    }

    #[test]
    fn test_payload_ptr_from_header() {
        let details = MessageTypeDetails::from::<i32, i32, i32>(TypeVariant::Dynamic);
        #[repr(C)]
        struct Demo {
            header: i32,
            _user_header: i32,
            payload: i32,
        }

        let demo = Demo {
            header: 123,
            _user_header: 123,
            payload: 9999,
        };

        let ptr: *const u8 = &demo.header as *const _ as *const u8;
        let payload_ptr = details.payload_ptr_from_header(ptr) as *const i32;
        let sut = unsafe { *payload_ptr };
        assert_that!(sut, eq demo.payload);
    }

    #[test]
    // test_sample_layout tests the sample layout for combinations of different types.
    fn test_sample_layout() {
        let details = MessageTypeDetails::from::<i64, i64, i64>(TypeVariant::FixedSize);
        let sut = details.sample_layout(0);
        #[cfg(target_pointer_width = "32")]
        let expected = 24;
        #[cfg(target_pointer_width = "64")]
        let expected = 32;
        assert_that!(sut.size(), eq expected);

        let details = MessageTypeDetails::from::<i64, i64, i64>(TypeVariant::FixedSize);
        let sut = details.sample_layout(2);
        #[cfg(target_pointer_width = "32")]
        let expected = 40;
        #[cfg(target_pointer_width = "64")]
        let expected = 48;
        assert_that!(sut.size(), eq expected);

        let details = MessageTypeDetails::from::<i64, i64, bool>(TypeVariant::FixedSize);
        let sut = details.sample_layout(3);
        #[cfg(target_pointer_width = "32")]
        let expected = 24;
        #[cfg(target_pointer_width = "64")]
        let expected = 32;
        assert_that!(sut.size(), eq expected);

        let details = MessageTypeDetails::from::<i64, i32, bool>(TypeVariant::FixedSize);
        let sut = details.sample_layout(11);

        #[cfg(target_pointer_width = "32")]
        let expected = 28;
        #[cfg(target_pointer_width = "64")]
        let expected = 32;
        assert_that!(sut.size(), eq expected);

        #[derive(ZeroCopySend)]
        #[repr(C)]
        struct Demo {
            _b: bool,
            _i16: i16,
            _i64: i64,
        }

        let details = MessageTypeDetails::from::<i64, i64, Demo>(TypeVariant::FixedSize);
        let sut = details.sample_layout(2);
        #[cfg(target_pointer_width = "32")]
        let expected = 48;
        #[cfg(target_pointer_width = "64")]
        let expected = 64;
        assert_that!(sut.size(), eq expected);
    }

    #[test]
    fn test_is_compatible_to_failed_when_types_differ() {
        let left = MessageTypeDetails::from::<i64, i64, i8>(TypeVariant::FixedSize);
        let right = MessageTypeDetails::from::<i64, i64, u8>(TypeVariant::FixedSize);
        let sut = left.is_compatible_to(&right);
        assert_that!(sut, eq false);

        let left = MessageTypeDetails::from::<i64, i64, i64>(TypeVariant::FixedSize);
        let right = MessageTypeDetails::from::<i64, i64, i32>(TypeVariant::FixedSize);
        let sut = left.is_compatible_to(&right);
        assert_that!(sut, eq false);
    }

    #[test]
    fn test_is_compatible_to_succeed_when_rhs_aligment_is_bigger() {
        let left = MessageTypeDetails::from::<i64, i64, i64>(TypeVariant::FixedSize);

        // right may have a different alignment from left.
        // but note that the header alignment must be the same
        let right = MessageTypeDetails {
            header: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: ALIGNMENT,
            },
            user_header: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: 2 * ALIGNMENT,
            },
            payload: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: 2 * ALIGNMENT,
            },
        };
        // smaller to bigger is allowed.
        let sut = left.is_compatible_to(&right);
        assert_that!(sut, eq true);

        // bigger to smaller is invalid.
        let sut = right.is_compatible_to(&left);
        assert_that!(sut, eq false);
    }

    #[test]
    fn test_is_compatible_to_fail_when_rhs_aligment_is_smaller() {
        let left = MessageTypeDetails::from::<i64, i64, i64>(TypeVariant::FixedSize);

        // right may have a different alignment from left.
        // but note that the header alignment must be the same
        let right = MessageTypeDetails {
            header: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: ALIGNMENT,
            },
            user_header: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: 2 * ALIGNMENT,
            },
            payload: TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: "i64".try_into().unwrap(),
                size: 8,
                alignment: 2 * ALIGNMENT,
            },
        };
        // bigger to smaller is invalid.
        let sut = right.is_compatible_to(&left);
        assert_that!(sut, eq false);
    }
}
