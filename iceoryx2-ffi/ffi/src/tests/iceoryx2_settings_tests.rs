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

use crate::tests::*;

#[test]
fn ffi_settings_are_equal_to_iceoryx2_settings() {
    assert_that!(IOX2_SERVICE_ID_LENGTH, eq iceoryx2::service::service_id::ServiceId::max_number_of_characters());
}
