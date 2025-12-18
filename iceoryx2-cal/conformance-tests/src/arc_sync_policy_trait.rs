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

use iceoryx2_bb_conformance_test_macros::conformance_test_module;

#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod arc_sync_policy_trait {
    use iceoryx2_bb_concurrency::atomic::Ordering;

    use iceoryx2_bb_concurrency::atomic::AtomicU64;
    use iceoryx2_bb_conformance_test_macros::conformance_test;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;

    #[conformance_test]
    pub fn create_and_locked_access_to_value_works<Sut: ArcSyncPolicy<AtomicU64>>() {
        let sut = Sut::new(AtomicU64::new(1234)).unwrap();
        sut.lock().store(4567, Ordering::Relaxed);
        assert_that!(sut.lock().load(Ordering::Relaxed), eq 4567);
    }

    #[conformance_test]
    pub fn create_and_guarded_access_to_value_works<Sut: ArcSyncPolicy<AtomicU64>>() {
        let sut = Sut::new(AtomicU64::new(987)).unwrap();
        let guard = sut.lock();
        guard.store(765, Ordering::Relaxed);
        assert_that!(guard.load(Ordering::Relaxed), eq 765);
    }

    #[conformance_test]
    pub fn has_arc_behavior_and_performs_shallow_copy<Sut: ArcSyncPolicy<AtomicU64>>() {
        let sut_1 = Sut::new(AtomicU64::new(5543)).unwrap();
        let sut_2 = sut_1.clone();

        sut_2.lock().store(1010101, Ordering::Relaxed);
        let value_1 = sut_1.lock().load(Ordering::Relaxed);
        let value_2 = sut_2.lock().load(Ordering::Relaxed);

        assert_that!(value_1, eq 1010101);
        assert_that!(value_2, eq 1010101);
    }

    #[conformance_test]
    pub fn uses_recursive_locking<Sut: ArcSyncPolicy<AtomicU64>>() {
        let sut = Sut::new(AtomicU64::new(55355)).unwrap();
        let guard_1 = sut.lock();
        let guard_2 = sut.lock();
        guard_1.store(33533, Ordering::Relaxed);
        assert_that!(guard_2.load(Ordering::Relaxed), eq 33533);
    }
}
