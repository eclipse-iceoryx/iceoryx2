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
    ipc,
    iceoryx2_conformance_tests::service_request_response,
    iceoryx2::service::ipc::Service
);

instantiate_conformance_tests_with_module!(
    local,
    iceoryx2_conformance_tests::service_request_response,
    iceoryx2::service::local::Service
);

instantiate_conformance_tests_with_module!(
    ipc_threadsafe,
    iceoryx2_conformance_tests::service_request_response,
    iceoryx2::service::ipc_threadsafe::Service
);

instantiate_conformance_tests_with_module!(
    local_threadsafe,
    iceoryx2_conformance_tests::service_request_response,
    iceoryx2::service::local_threadsafe::Service
);
