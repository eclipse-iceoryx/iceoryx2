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

use core::marker::PhantomData;

use iceoryx2_bb_flatbuffers::type_name;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

struct TypeWithoutNamespace {}

#[test]
pub fn type_without_extra_namespace_works() {
    let sut = type_name::<TypeWithoutNamespace>();

    assert_that!(sut.name, eq "TypeWithoutNamespace");
}

pub mod some_namespace {
    pub struct TypeWithNamespace {}
}

#[test]
pub fn type_with_extra_namespace_works() {
    let sut = type_name::<some_namespace::TypeWithNamespace>();

    assert_that!(sut.name, eq "TypeWithNamespace");
    assert_that!(sut.namespace, eq "some_namespace");
}

struct TypeWithLifetimeArg<'a> {
    _data: PhantomData<&'a ()>,
}

#[test]
pub fn type_with_lifetime_arg_works() {
    let sut = type_name::<TypeWithLifetimeArg<'static>>();

    assert_that!(sut.name, eq "TypeWithLifetimeArg");
}

pub mod another_namespace {
    use super::*;
    pub struct TypeWithLifetimeArgAndNamespace<'a> {
        _data: PhantomData<&'a ()>,
    }
}

#[test]
pub fn type_with_lifetime_arg_and_extra_namespace_works() {
    let sut = type_name::<another_namespace::TypeWithLifetimeArgAndNamespace<'static>>();

    assert_that!(sut.name, eq "TypeWithLifetimeArgAndNamespace");
    assert_that!(sut.namespace, eq "another_namespace");
}
