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

use iceoryx2_bb_system_types::ipv4_address::{Ipv4Address, BROADCAST, LOCALHOST, UNSPECIFIED};
use iceoryx2_bb_testing::assert_that;

#[test]
fn ipv4_address_is_created_correctly() {
    let sut = Ipv4Address::new(1, 2, 3, 4).octets();

    assert_that!(sut[0], eq  1);
    assert_that!(sut[1], eq  2);
    assert_that!(sut[2], eq  3);
    assert_that!(sut[3], eq  4);
}

#[test]
fn ipv4_address_is_unspecified_works() {
    assert_that!(UNSPECIFIED.is_unspecified(), eq true);
    assert_that!(LOCALHOST.is_unspecified(), eq false);
}

#[test]
fn ipv4_address_is_loopback_works() {
    assert_that!(Ipv4Address::new(127,0,0,0).is_loopback(), eq true);
    assert_that!(Ipv4Address::new(127, 255, 255, 255).is_loopback(), eq true);
    assert_that!(UNSPECIFIED.is_loopback(), eq false);
}

#[test]
fn ipv4_address_is_private_works() {
    assert_that!(Ipv4Address::new(10,1,2,3).is_private(), eq true);
    assert_that!(Ipv4Address::new(192, 168, 1, 2).is_private(), eq true);
    assert_that!(Ipv4Address::new(172, 16, 1, 2).is_private(), eq true);
    assert_that!(BROADCAST.is_private(), eq false);
}

#[test]
fn ipv4_address_is_link_local_works() {
    assert_that!(Ipv4Address::new(169,254,2,3).is_link_local(), eq true);
    assert_that!(BROADCAST.is_link_local(), eq false);
}

#[test]
fn ipv4_address_is_shared_works() {
    assert_that!(Ipv4Address::new(100,64,1,2).is_shared(), eq true);
    assert_that!(UNSPECIFIED.is_shared(), eq false);
}

#[test]
fn ipv4_address_is_benchmarking_works() {
    assert_that!(Ipv4Address::new(198,18,1,2).is_benchmarking(), eq true);
    assert_that!(LOCALHOST.is_benchmarking(), eq false);
}

#[test]
fn ipv4_address_is_reserved_works() {
    assert_that!(Ipv4Address::new(244,18,1,2).is_reserved(), eq true);
    assert_that!(BROADCAST.is_reserved(), eq false);
}

#[test]
fn ipv4_address_is_multicast_works() {
    assert_that!(Ipv4Address::new(224,18,1,2).is_multicast(), eq true);
    assert_that!(BROADCAST.is_multicast(), eq false);
}

#[test]
fn ipv4_address_is_broadcast_works() {
    assert_that!(BROADCAST.is_broadcast(), eq true);
    assert_that!(Ipv4Address::new(224,18,1,2).is_broadcast(), eq false);
}

#[test]
fn ipv4_address_is_documentation_works() {
    assert_that!(Ipv4Address::new(192,0,2,2).is_documentation(), eq true);
    assert_that!(BROADCAST.is_documentation(), eq false);
}

#[test]
fn ipv4_address_is_global_works() {
    assert_that!(Ipv4Address::new(92,0,2,2).is_global(), eq true);
    assert_that!(BROADCAST.is_global(), eq false);
    assert_that!(LOCALHOST.is_global(), eq false);
    assert_that!(UNSPECIFIED.is_global(), eq false);
}
