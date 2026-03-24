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
    unix_datagram,
    iceoryx2_cal_conformance_tests::communication_channel_trait,
    iceoryx2_cal::communication_channel::unix_datagram::Channel<u64>
);

instantiate_conformance_tests_with_module!(
    posix_shared_memory,
    iceoryx2_cal_conformance_tests::communication_channel_trait,
    iceoryx2_cal::communication_channel::posix_shared_memory::Channel
);

instantiate_conformance_tests_with_module!(
    process_local,
    iceoryx2_cal_conformance_tests::communication_channel_trait,
    iceoryx2_cal::communication_channel::process_local::Channel
);
