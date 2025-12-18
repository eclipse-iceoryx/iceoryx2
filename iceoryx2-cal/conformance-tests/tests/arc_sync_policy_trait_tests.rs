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

use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_testing::instantiate_conformance_tests_with_module;
use iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected;
use iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded;

instantiate_conformance_tests_with_module!(
    mutex_protected,
    iceoryx2_cal_conformance_tests::arc_sync_policy_trait,
    super::MutexProtected<super::AtomicU64>
);

instantiate_conformance_tests_with_module!(
    single_threaded,
    iceoryx2_cal_conformance_tests::arc_sync_policy_trait,
    super::SingleThreaded<super::AtomicU64>
);
