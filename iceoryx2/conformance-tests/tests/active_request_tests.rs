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

use iceoryx2_conformance_tests::active_request;
use iceoryx2_conformance_tests::active_request_tests;

mod ipc {
    use super::*;
    active_request_tests!(iceoryx2::service::ipc::Service);
}

mod local {
    use super::*;
    active_request_tests!(iceoryx2::service::local::Service);
}

mod ipc_threadsafe {
    use super::*;
    active_request_tests!(iceoryx2::service::ipc_threadsafe::Service);
}

mod local_threadsafe {
    use super::*;
    active_request_tests!(iceoryx2::service::local_threadsafe::Service);
}
