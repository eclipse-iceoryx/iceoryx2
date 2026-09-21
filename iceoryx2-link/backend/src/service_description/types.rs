// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use alloc::string::String;
use core::alloc::Layout;

use iceoryx2::service::header::publish_subscribe::Header;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_log::{fail, fatal_panic, origin};
use serde::{Deserialize, Serialize};

/// The types of a service's pattern.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServiceTypes {
    PublishSubscribe(SampleTypes),
    /// An event service has no types, a notification carries an id.
    Event,
}

impl ServiceTypes {
    /// The publish-subscribe types. Panics on another pattern, the caller
    /// is responsible for ensuring the correct variant is provided.
    pub fn publish_subscribe(&self) -> &SampleTypes {
        let origin = origin!("ServiceTypes::publish_subscribe");

        match self {
            ServiceTypes::PublishSubscribe(types) => types,
            ServiceTypes::Event => fatal_panic!(
                from origin,
                "Tried to convert service types of another pattern to publish-subscribe types"
            ),
        }
    }
}

/// The types of a sample's header and payload.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SampleTypes {
    pub payload: TypeDescription,
    pub user_header: TypeDescription,
}

impl SampleTypes {
    /// The type details of the payload and the user header, checked to
    /// compose with `iceoryx2`'s own header into a valid sample layout.
    pub fn type_details(&self) -> Result<(TypeDetail, TypeDetail), InvalidSampleLayout> {
        let origin = origin!("SampleTypes::type_details");

        let payload = match TypeDetail::try_from(&self.payload) {
            Ok(detail) => detail,
            Err(error) => {
                fail!(
                    from origin,
                    with InvalidSampleLayout::Payload(error),
                    "Payload type '{}' cannot be represented as a type detail", self.payload.type_name
                );
            }
        };
        let user_header = match TypeDetail::try_from(&self.user_header) {
            Ok(detail) => detail,
            Err(error) => {
                fail!(
                    from origin,
                    with InvalidSampleLayout::UserHeader(error),
                    "User header type '{}' cannot be represented as a type detail", self.user_header.type_name
                );
            }
        };

        // A sample is the header, the user header and the payload laid out
        // in that order. The service builds that layout unchecked, so
        // check here that it fits within what a layout may hold.
        let composed = Layout::new::<Header>()
            .extend(layout_of(&user_header))
            .and_then(|(layout, _)| layout.extend(layout_of(&payload)));

        fail!(
            from origin,
            when composed,
            with InvalidSampleLayout::Overflow,
            "Samples of '{}' under '{}' exceed what a layout may hold",
            self.payload.type_name, self.user_header.type_name
        );
        Ok((payload, user_header))
    }
}

/// The layout of one type detail, whose size and alignment
/// [`TypeDetail::try_from`] checked to form one.
fn layout_of(detail: &TypeDetail) -> Layout {
    Layout::from_size_align(detail.size(), detail.alignment()).expect("checked by try_from")
}

/// The sample layout a publish-subscribe service of some types cannot be
/// created with.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum InvalidSampleLayout {
    Payload(InvalidTypeDescription),
    UserHeader(InvalidTypeDescription),
    /// The header, user header and payload together exceed what a layout
    /// may hold.
    Overflow,
}

impl core::fmt::Display for InvalidSampleLayout {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InvalidSampleLayout::{self:?}")
    }
}

impl core::error::Error for InvalidSampleLayout {}

/// A [`TypeDetail`] in a form that crosses the boundary.
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TypeDescription {
    pub variant: TypeVariant,
    pub type_name: String,
    pub size: usize,
    pub alignment: usize,
}

impl From<&TypeDetail> for TypeDescription {
    fn from(detail: &TypeDetail) -> Self {
        Self {
            variant: detail.variant(),
            type_name: String::from_utf8_lossy(detail.type_name()).into_owned(),
            size: detail.size(),
            alignment: detail.alignment(),
        }
    }
}

/// A [`TypeDescription`] received from the opposing side that cannot be a
/// valid [`TypeDetail`].
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum InvalidTypeDescription {
    TypeNameTooLong,
    AlignmentNotPowerOfTwo,
    SizeNotMultipleOfAlignment,
    /// The size rounded up to the alignment exceeds what a layout may hold.
    LayoutOverflow,
}

impl core::fmt::Display for InvalidTypeDescription {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InvalidTypeDescription::{self:?}")
    }
}

impl core::error::Error for InvalidTypeDescription {}

impl TryFrom<&TypeDescription> for TypeDetail {
    type Error = InvalidTypeDescription;

    fn try_from(description: &TypeDescription) -> Result<Self, Self::Error> {
        let origin = origin!("TypeDetail::try_from(&TypeDescription)");

        if !description.alignment.is_power_of_two() {
            return Err(InvalidTypeDescription::AlignmentNotPowerOfTwo);
        }
        if !description.size.is_multiple_of(description.alignment) {
            return Err(InvalidTypeDescription::SizeNotMultipleOfAlignment);
        }
        if Layout::from_size_align(description.size, description.alignment).is_err() {
            return Err(InvalidTypeDescription::LayoutOverflow);
        }
        let detail = fail!(
            from origin,
            when TypeDetail::__internal_new_from_parts(
                description.variant,
                &description.type_name,
                description.size,
                description.alignment,
            ),
            with InvalidTypeDescription::TypeNameTooLong,
            "Type name '{}' exceeds the maximum length", description.type_name
        );
        Ok(detail)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::constants::MAX_TYPE_NAME_LENGTH;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn type_detail_round_trips() {
        let detail = TypeDetail::new::<u64>(TypeVariant::FixedSize);

        let description = TypeDescription::from(&detail);
        assert_that!(description.variant, eq TypeVariant::FixedSize);
        assert_that!(description.type_name, eq "u64");
        assert_that!(description.size, eq core::mem::size_of::<u64>());
        assert_that!(description.alignment, eq core::mem::align_of::<u64>());

        let round_tripped = TypeDetail::try_from(&description).expect("valid description");
        assert_that!(round_tripped, eq detail);
    }

    #[test]
    fn overlong_type_name_is_rejected() {
        const SIZE: usize = 1;
        const ALIGNMENT: usize = 1;

        let description = TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: "x".repeat(MAX_TYPE_NAME_LENGTH + 1),
            size: SIZE,
            alignment: ALIGNMENT,
        };

        let result = TypeDetail::try_from(&description);
        assert_that!(result, eq Err(InvalidTypeDescription::TypeNameTooLong));
    }

    #[test]
    fn alignment_that_is_not_a_power_of_two_is_rejected() {
        const NOT_POWERS_OF_TWO: [usize; 3] = [0, 3, 6];
        const SIZE: usize = 12;

        for alignment in NOT_POWERS_OF_TWO {
            let description = TypeDescription {
                variant: TypeVariant::FixedSize,
                type_name: "test_type".into(),
                size: SIZE,
                alignment,
            };

            let result = TypeDetail::try_from(&description);
            assert_that!(result, eq Err(InvalidTypeDescription::AlignmentNotPowerOfTwo));
        }
    }

    #[test]
    fn size_that_is_not_a_multiple_of_alignment_is_rejected() {
        const SIZE: usize = 6;
        const ALIGNMENT: usize = 4;

        let description = TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: "test_type".into(),
            size: SIZE,
            alignment: ALIGNMENT,
        };

        let result = TypeDetail::try_from(&description);
        assert_that!(
            result,
            eq Err(InvalidTypeDescription::SizeNotMultipleOfAlignment)
        );
    }

    // No real type has these sizes, a description received from the
    // opposing side can. They sit at the bound `Layout::from_size_align`
    // enforces, a size that fits an `isize` once rounded up to the
    // alignment, so one step under passes and the step itself fails.

    /// The largest power of two a layout holds as size and alignment.
    const LARGEST_WITHIN_LAYOUT_BOUND: usize = (usize::MAX >> 2) + 1;
    /// The first size a layout refuses.
    const FIRST_BEYOND_LAYOUT_BOUND: usize = (usize::MAX >> 1) + 1;

    fn described(type_name: &str, size: usize, alignment: usize) -> TypeDescription {
        TypeDescription {
            variant: TypeVariant::FixedSize,
            type_name: type_name.into(),
            size,
            alignment,
        }
    }

    #[test]
    fn size_beyond_what_a_layout_holds_is_rejected() {
        let description = described("huge", FIRST_BEYOND_LAYOUT_BOUND, FIRST_BEYOND_LAYOUT_BOUND);

        let result = TypeDetail::try_from(&description);
        assert_that!(result, eq Err(InvalidTypeDescription::LayoutOverflow));
    }

    #[test]
    fn ordinary_types_compose_into_a_sample_layout() {
        let types = SampleTypes {
            payload: TypeDescription::from(&TypeDetail::new::<u64>(TypeVariant::FixedSize)),
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
        };

        let result = types.type_details();
        assert_that!(result.is_ok(), eq true);
    }

    #[test]
    fn a_large_payload_within_the_bound_composes_into_a_sample_layout() {
        // Whether such a service can be created is the allocator's call
        // at open, the layout itself is valid.
        const GIGABYTE: usize = 1 << 30;
        const SIZE: usize = GIGABYTE;
        const ALIGNMENT: usize = core::mem::align_of::<u64>();

        let types = SampleTypes {
            payload: described("large", SIZE, ALIGNMENT),
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
        };

        let result = types.type_details();
        assert_that!(result.is_ok(), eq true);
    }

    #[test]
    fn types_whose_sample_layout_overflows_are_rejected() {
        // Valid on its own, the headers rounded up to its alignment plus
        // one element exceed what a layout may hold.
        let payload = described(
            "huge",
            LARGEST_WITHIN_LAYOUT_BOUND,
            LARGEST_WITHIN_LAYOUT_BOUND,
        );
        assert_that!(TypeDetail::try_from(&payload).is_ok(), eq true);
        let types = SampleTypes {
            payload,
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
        };

        let result = types.type_details();
        assert_that!(result, eq Err(InvalidSampleLayout::Overflow));
    }

    #[test]
    fn an_invalid_payload_type_names_the_payload() {
        let types = SampleTypes {
            payload: described("huge", FIRST_BEYOND_LAYOUT_BOUND, FIRST_BEYOND_LAYOUT_BOUND),
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
        };

        let result = types.type_details();
        assert_that!(
            result,
            eq Err(InvalidSampleLayout::Payload(InvalidTypeDescription::LayoutOverflow))
        );
    }
}
