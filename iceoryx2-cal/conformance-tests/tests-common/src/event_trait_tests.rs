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
    process_local_socket_pair,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::process_local_socketpair::EventImpl
);

instantiate_conformance_tests_with_module!(
    unix_datagram,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::unix_datagram_socket::EventImpl
);

instantiate_conformance_tests_with_module!(
    sem_bitset_process_local,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::sem_bitset_process_local::Event
);

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
instantiate_conformance_tests_with_module!(
    sem_bitset_posix_shared_memory,
    iceoryx2_cal_conformance_tests::event_trait,
    iceoryx2_cal::event::sem_bitset_posix_shared_memory::Event
);
