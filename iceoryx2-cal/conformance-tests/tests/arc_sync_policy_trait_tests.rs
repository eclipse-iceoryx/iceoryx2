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

mod mutex_protected {
    super::instantiate_conformance_tests!(
        iceoryx2_cal_conformance_tests::arc_sync_policy_trait,
        iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected<
        iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64,
        >
    );
}

mod single_threaded {
    super::instantiate_conformance_tests!(
        iceoryx2_cal_conformance_tests::arc_sync_policy_trait,
        iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<
        iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64,
        >
    );
}
