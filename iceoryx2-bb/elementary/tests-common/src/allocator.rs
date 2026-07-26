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

use iceoryx2_bb_elementary_traits::allocator::{AllocationError, AllocationGrowError};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn allocation_error_to_allocation_grow_error_conversion_works() {
    assert_that!(Into::<AllocationGrowError>::into(AllocationError::AlignmentFailure),
        eq AllocationGrowError::AlignmentFailure);

    assert_that!(Into::<AllocationGrowError>::into(AllocationError::InternalError),
        eq AllocationGrowError::InternalError);

    assert_that!(Into::<AllocationGrowError>::into(AllocationError::OutOfMemory),
        eq AllocationGrowError::OutOfMemory);

    assert_that!(Into::<AllocationGrowError>::into(AllocationError::SizeIsZero),
        eq AllocationGrowError::SizeIsZero);

    assert_that!(Into::<AllocationGrowError>::into(AllocationError::SizeTooLarge),
        eq AllocationGrowError::InternalError);
}
