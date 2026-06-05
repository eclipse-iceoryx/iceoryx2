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

use iceoryx2_bb_testing::instantiate_conformance_tests_with_module;

// semaphore is implemented in the platform abstraction layer with WaitOnAddress
// on Windows that comes with some problems
#[cfg(not(target_os = "windows"))]
instantiate_conformance_tests_with_module!(
    semaphore_shared_memory_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::bit_set::RelocatableBitSet,
    iceoryx2_cal::event::SemaphoreShmBitSet
);

instantiate_conformance_tests_with_module!(
    unix_datagram_shared_memory_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::bit_set::RelocatableBitSet,
    iceoryx2_cal::event::UnixDatagramShmBitSet
);

instantiate_conformance_tests_with_module!(
    socket_pair_process_local_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::bit_set::RelocatableBitSet,
    iceoryx2_cal::event::SocketPairBitSet
);

// semaphore is implemented in the platform abstraction layer with WaitOnAddress
// on Windows that comes with some problems
#[cfg(not(target_os = "windows"))]
instantiate_conformance_tests_with_module!(
    semaphore_shared_memory_counting_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::counting_bit_set::RelocatableCountingBitSet,
    iceoryx2_cal::event::SemaphoreShmCountingBitSet
);

instantiate_conformance_tests_with_module!(
    unix_datagram_shared_memory_counting_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::counting_bit_set::RelocatableCountingBitSet,
    iceoryx2_cal::event::UnixDatagramShmCountingBitSet
);

instantiate_conformance_tests_with_module!(
    socket_pair_process_local_counting_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::counting_bit_set::RelocatableCountingBitSet,
    iceoryx2_cal::event::SocketPairCountingBitSet
);
