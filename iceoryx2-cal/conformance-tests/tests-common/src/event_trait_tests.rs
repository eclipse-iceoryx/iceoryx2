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

instantiate_conformance_tests_with_module!(
    semaphore_shared_memory_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::bit_set::RelocatableBitSet,
    iceoryx2_cal::event::trigger::semaphore::SemaphoreShmBitSet
);

instantiate_conformance_tests_with_module!(
    unix_datagram_shared_memory_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::bit_set::RelocatableBitSet,
    iceoryx2_cal::event::trigger::unix_datagram_socket::UnixDatagramShmBitSet
);

instantiate_conformance_tests_with_module!(
    socket_pair_process_local_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::bit_set::RelocatableBitSet,
    iceoryx2_cal::event::trigger::socket_pair::SocketPairBitSet
);

instantiate_conformance_tests_with_module!(
    semaphore_shared_memory_counting_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::counting_bit_set::RelocatableCountingBitSet,
    iceoryx2_cal::event::trigger::semaphore::SemaphoreShmCountingBitSet
);

instantiate_conformance_tests_with_module!(
    unix_datagram_shared_memory_counting_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::counting_bit_set::RelocatableCountingBitSet,
    iceoryx2_cal::event::trigger::unix_datagram_socket::UnixDatagramShmCountingBitSet
);

instantiate_conformance_tests_with_module!(
    socket_pair_process_local_counting_bitset,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::event_state::counting_bit_set::RelocatableCountingBitSet,
    iceoryx2_cal::event::trigger::socket_pair::SocketPairCountingBitSet
);
