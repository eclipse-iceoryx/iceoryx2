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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod clock_trait {
    use iceoryx2_bb_posix::testing::generate_file_path;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::clock::Clock;
    use iceoryx2_cal::testing::*;

    #[conformance_test]
    pub fn whatever<Sut: Clock>() {
        let name = generate_file_path().file_name();
        //let config = generate_isolated_config::<Sut>();

        assert_that!(true, eq true);
    }
}
