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
use iceoryx2_conformance_tests::service::service::{
    BlackboardTests, EventTests, PubSubTests, RequestResponseTests,
};

mod ipc {
    use super::*;
    use iceoryx2::service::ipc::Service;

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::EventTests::<super::Service>
        );
    }

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::PubSubTests::<super::Service>
        );
    }

    mod request_response {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::RequestResponseTests::<super::Service>
        );
    }

    mod blackboard {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::BlackboardTests::<super::Service>
        );
    }
}

mod local {
    use super::*;
    use iceoryx2::service::local::Service;

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::EventTests::<super::Service>
        );
    }

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::PubSubTests::<super::Service>
        );
    }

    mod request_response {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::RequestResponseTests::<super::Service>
        );
    }

    mod blackboard {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::BlackboardTests::<super::Service>
        );
    }
}

mod ipc_threadsafe {
    use super::*;
    use iceoryx2::service::ipc_threadsafe::Service;

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::EventTests::<super::Service>
        );
    }

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::PubSubTests::<super::Service>
        );
    }

    mod request_response {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::RequestResponseTests::<super::Service>
        );
    }

    mod blackboard {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::BlackboardTests::<super::Service>
        );
    }
}

mod local_threadsafe {
    use super::*;
    use iceoryx2::service::local_threadsafe::Service;

    mod event {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::EventTests::<super::Service>
        );
    }

    mod publish_subscribe {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::PubSubTests::<super::Service>
        );
    }

    mod request_response {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::RequestResponseTests::<super::Service>
        );
    }

    mod blackboard {
        use super::*;

        instantiate_conformance_tests!(
            iceoryx2_conformance_tests::service,
            super::Service,
            super::BlackboardTests::<super::Service>
        );
    }
}
