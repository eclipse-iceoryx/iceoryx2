mod alt_1 {
    use iceoryx2_conformance_tests::active_request;
    use iceoryx2_conformance_tests::active_request_tests;
    use iceoryx2_conformance_tests::node;
    use iceoryx2_conformance_tests::node_tests;
    // can also be used with just 'use iceoryx2_conformance_tests::*;'; but
    // a module is strictly required even if only one single test is instantiated

    mod ipc {
        use super::*; // this is required
        active_request_tests!(iceoryx2::service::ipc::Service);
        node_tests!(iceoryx2::service::ipc::Service);
    }

    mod local {
        use super::*;
        active_request_tests!(iceoryx2::service::ipc::Service);
        node_tests!(iceoryx2::service::local::Service);
    }

    mod ipc_threadsafe {
        use super::*;
        active_request_tests!(iceoryx2::service::ipc::Service);
        node_tests!(iceoryx2::service::ipc_threadsafe::Service);
    }

    mod local_threadsafe {
        use super::*;
        active_request_tests!(iceoryx2::service::ipc::Service);
        node_tests!(iceoryx2::service::local_threadsafe::Service);
    }
}
mod alt_2 {
    // modules only required when the same test is instantiated multiple times

    mod ipc {
        use iceoryx2_conformance_tests::*;
        active_request_tests_alt!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::ipc::Service
        );
        node_tests_alt!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::ipc::Service
        );
    }

    mod local {
        use iceoryx2_conformance_tests::*;
        active_request_tests_alt!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::local::Service
        );
        node_tests_alt!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::local::Service
        );
    }

    mod ipc_threadsafe {
        use iceoryx2_conformance_tests::*;
        active_request_tests_alt!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::ipc_threadsafe::Service
        );
        node_tests_alt!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::ipc_threadsafe::Service
        );
    }

    mod local_threadsafe {
        use iceoryx2_conformance_tests::*;
        active_request_tests_alt!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::local_threadsafe::Service
        );
        node_tests_alt!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::local_threadsafe::Service
        );
    }
}

mod alt_3 {
    // modules only required when the same test is instantiated multiple times

    use iceoryx2_bb_testing::instantiate_conformance_tests;

    mod ipc {
        // instead of 'super::', 'iceoryx2_bb_testing::' could be used
        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::ipc::Service
        );

        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::ipc::Service
        );
    }
    mod local {
        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::local::Service
        );

        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::local::Service
        );
    }
    mod ipc_threadsafe {
        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::ipc_threadsafe::Service
        );

        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::ipc_threadsafe::Service
        );
    }
    mod local_threadsafe {
        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::active_request,
            iceoryx2::service::local_threadsafe::Service
        );

        super::instantiate_conformance_tests!(
            iceoryx2_conformance_tests::node,
            iceoryx2::service::local_threadsafe::Service
        );
    }
}
