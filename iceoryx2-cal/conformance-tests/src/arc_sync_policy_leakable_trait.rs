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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod arc_sync_policy_leakable_trait {
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::leak_tracker::LeakTracker;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;

    #[conformance_test]
    pub fn create_and_locked_access_to_value_works<Sut: ArcSyncPolicy<LeakTracker>>() {
        let tracker = LeakTracker::start_tracking();
        let sut = Sut::new(LeakTracker::new()).unwrap();

        assert_that!(tracker.creation_count(), eq 1);
        assert_that!(tracker.drop_count(), eq 0);
        assert_that!(tracker.leak_count(), eq 0);

        Sut::abandon(sut);

        assert_that!(tracker.creation_count(), eq 1);
        assert_that!(tracker.drop_count(), eq 0);
        assert_that!(tracker.leak_count(), eq 1);
    }
}
