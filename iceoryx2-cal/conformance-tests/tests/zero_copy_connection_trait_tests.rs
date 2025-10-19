// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_testing::instantiate_conformance_tests;

mod posix_shared_memory {
    use super::*;
    instantiate_conformance_tests!(
        iceoryx2_cal_conformance_tests::zero_copy_connection_trait,
        iceoryx2_cal::zero_copy_connection::posix_shared_memory::Connection
    );
}

mod process_local {
    use super::*;
    instantiate_conformance_tests!(
        iceoryx2_cal_conformance_tests::zero_copy_connection_trait,
        iceoryx2_cal::zero_copy_connection::process_local::Connection
    );
}
