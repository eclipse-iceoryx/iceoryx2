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

use iceoryx2_bb_elementary::unique_id::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn unique_id_is_unique() {
    let a = UniqueId::new();
    let b = UniqueId::new();
    let c = UniqueId::new();

    assert_that!(a, ne b);
    assert_that!(a, ne c);
    assert_that!(b, ne c);
}

#[test]
fn typed_unique_id_is_unique() {
    let a = TypedUniqueId::<u64>::new();
    let b = TypedUniqueId::<u64>::new();
    let c = TypedUniqueId::<u64>::new();

    assert_that!(a, ne b);
    assert_that!(a, ne c);
    assert_that!(b, ne c);

    let d = TypedUniqueId::<u32>::new();
    assert_that!(a.value(), ne d.value());
}
