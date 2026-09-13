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

use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_log::{fail, fatal_panic};
use serde::{Deserialize, Serialize};

/// The types of a service's pattern.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ServiceTypes {
    PublishSubscribe(PublishSubscribeTypes),
    /// An event service has no types, a notification carries an id.
    Event,
}

impl ServiceTypes {
    /// The publish-subscribe types. Panics on another pattern, the caller
    /// is responsible for ensuring the correct variant is provided.
    pub fn publish_subscribe(&self) -> &PublishSubscribeTypes {
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

/// The types of a publish-subscribe service's data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PublishSubscribeTypes {
    pub payload: TypeDescription,
    pub user_header: TypeDescription,
}

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
}
