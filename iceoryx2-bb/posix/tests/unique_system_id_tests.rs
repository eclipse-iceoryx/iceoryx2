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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::unique_system_id_tests;

#[test]
fn unique_system_id_is_unique() {
    unique_system_id_tests::unique_system_id_is_unique();
}

#[test]
fn unique_system_id_concurrently_created_ids_are_unique() {
    unique_system_id_tests::unique_system_id_concurrently_created_ids_are_unique();
}
