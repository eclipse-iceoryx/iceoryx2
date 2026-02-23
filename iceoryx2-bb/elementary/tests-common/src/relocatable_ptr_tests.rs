// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_elementary_traits::pointer_trait::PointerTrait;
use iceoryx2_bb_testing::assert_that;

#[test]
fn relocatable_pointer_works() {
    let mut sut = RelocatablePointer::<i32>::new(0);
    let _o2: i32 = 0;
    let mut _o3: i32 = 0;
    let value = 9128391;

    let distance = core::ptr::addr_of!(_o3) as isize - core::ptr::addr_of!(sut) as isize;

    sut = RelocatablePointer::<i32>::new(distance);
    _o3 = value;
    assert_that!(unsafe { *sut.as_ptr() }, eq value);
}
