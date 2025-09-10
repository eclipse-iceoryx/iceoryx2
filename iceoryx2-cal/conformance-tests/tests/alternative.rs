mod alt_1 {
    use iceoryx2_cal_conformance_tests::arc_sync_policy_trait;
    use iceoryx2_cal_conformance_tests::arc_sync_policy_trait_tests;
    use iceoryx2_cal_conformance_tests::serialize_trait;
    use iceoryx2_cal_conformance_tests::serialize_trait_tests;
    // can also be used with just 'use iceoryx2_cal_conformance_tests::*;'; but
    // a module is strictly required even if only one single test is instantiated

    mod mutex_protected {
        use super::*; // this is required
        arc_sync_policy_trait_tests!(
            iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected<
            iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64,
            >
        );
    }

    mod single_threaded {
        use super::*;
        arc_sync_policy_trait_tests!(
            iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<
            iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64,
            >
        );
    }

    mod toml {
        use super::*;
        serialize_trait_tests!(iceoryx2_cal::serialize::toml::Toml);
    }

    mod cdr {
        use super::*;
        serialize_trait_tests!(iceoryx2_cal::serialize::cdr::Cdr);
    }

    mod postcard {
        use super::*;
        serialize_trait_tests!(iceoryx2_cal::serialize::postcard::Postcard);
    }
}

mod alt_2 {
    // modules only required when the same test is instantiated multiple times

    mod mutex_protected {
        use iceoryx2_cal_conformance_tests::*;
        arc_sync_policy_trait_tests_alt!(
            iceoryx2_cal_conformance_tests::arc_sync_policy_trait,
            iceoryx2_cal::arc_sync_policy::mutex_protected::MutexProtected<
            iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64,
            >
        );
    }

    mod single_threaded {
        use iceoryx2_cal_conformance_tests::*;
        arc_sync_policy_trait_tests_alt!(
            iceoryx2_cal_conformance_tests::arc_sync_policy_trait,
            iceoryx2_cal::arc_sync_policy::single_threaded::SingleThreaded<
            iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64,
            >
        );
    }

    mod toml {
        use iceoryx2_cal_conformance_tests::*;
        serialize_trait_tests_alt!(
            iceoryx2_cal_conformance_tests::serialize_trait,
            iceoryx2_cal::serialize::toml::Toml
        );
    }

    mod cdr {
        use iceoryx2_cal_conformance_tests::*;
        serialize_trait_tests_alt!(
            iceoryx2_cal_conformance_tests::serialize_trait,
            iceoryx2_cal::serialize::cdr::Cdr
        );
    }

    mod postcard {
        use iceoryx2_cal_conformance_tests::*;
        serialize_trait_tests_alt!(
            iceoryx2_cal_conformance_tests::serialize_trait,
            iceoryx2_cal::serialize::postcard::Postcard
        );
    }
}

mod alt_3 {
    // modules only required when the same test is instantiated multiple times

    iceoryx2_bb_testing::instantiate_conformance_tests;

    mod mutex_protected {
        // instead of 'super::', 'iceoryx2_bb_testing::' could be used
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

    mod toml {
        super::instantiate_conformance_tests!(
            iceoryx2_cal_conformance_tests::serialize_trait,
            iceoryx2_cal::serialize::toml::Toml
        );
    }

    mod cdr {
        super::instantiate_conformance_tests!(
            iceoryx2_cal_conformance_tests::serialize_trait,
            iceoryx2_cal::serialize::cdr::Cdr
        );
    }

    mod postcard {
        super::instantiate_conformance_tests!(
            iceoryx2_cal_conformance_tests::serialize_trait,
            iceoryx2_cal::serialize::postcard::Postcard
        );
    }
}
