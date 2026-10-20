// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod trigger_trait {
    use alloc::format;
    use alloc::vec;

    use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::reactor::{Reactor, ReactorBuilder, posix_select};
    use iceoryx2_cal::testing::create_test_clock;
    use iceoryx2_cal::trigger::*;

    const NUMBER_OF_ACTIVATIONS: usize = 10000;

    fn create_reactor() -> posix_select::Reactor {
        <posix_select::Reactor as Reactor>::Builder::new()
            .create(create_test_clock())
            .unwrap()
    }

    #[conformance_test]
    pub fn reset_without_activation_reports_no_activation<Sut: Trigger>() {
        let (_activator, observer) = Sut::Builder::new().create().unwrap();

        assert_that!(observer.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn reset_after_activation_reports_the_activation_once<Sut: Trigger>() {
        let (activator, observer) = Sut::Builder::new().create().unwrap();

        assert_that!(activator.activate(), is_ok);

        assert_that!(observer.reset(), eq Ok(true));
        assert_that!(observer.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn many_activations_before_a_reset_are_reported_as_one<Sut: Trigger>() {
        let (activator, observer) = Sut::Builder::new().create().unwrap();

        for _ in 0..NUMBER_OF_ACTIVATIONS {
            assert_that!(activator.activate(), is_ok);
        }

        assert_that!(observer.reset(), eq Ok(true));
        assert_that!(observer.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn trigger_can_be_activated_again_after_a_reset<Sut: Trigger>() {
        let (activator, observer) = Sut::Builder::new().create().unwrap();

        for _ in 0..3 {
            assert_that!(activator.activate(), is_ok);
            assert_that!(observer.reset(), eq Ok(true));
            assert_that!(observer.reset(), eq Ok(false));
        }
    }

    #[conformance_test]
    pub fn triggers_do_not_activate_each_other<Sut: Trigger>() {
        let (activator_1, observer_1) = Sut::Builder::new().create().unwrap();
        let (_activator_2, observer_2) = Sut::Builder::new().create().unwrap();

        assert_that!(activator_1.activate(), is_ok);

        assert_that!(observer_2.reset(), eq Ok(false));
        assert_that!(observer_1.reset(), eq Ok(true));
    }

    #[conformance_test]
    pub fn activated_observer_wakes_up_reactor<Sut: Trigger>() {
        let (activator, observer) = Sut::Builder::new().create().unwrap();
        let reactor = create_reactor();
        let _guard = reactor.attach(&observer).unwrap();

        assert_that!(activator.activate(), is_ok);

        let mut triggered_fds = vec![];
        let result = reactor.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() }));

        assert_that!(result, eq Ok(1));
        assert_that!(triggered_fds, eq vec![unsafe { observer.file_descriptor().native_handle() }]);
    }

    #[conformance_test]
    pub fn observer_without_activation_does_not_wake_up_reactor<Sut: Trigger>() {
        let (_activator, observer) = Sut::Builder::new().create().unwrap();
        let reactor = create_reactor();
        let _guard = reactor.attach(&observer).unwrap();

        assert_that!(reactor.try_wait(|_| {}), eq Ok(0));
    }

    #[conformance_test]
    pub fn reset_observer_does_not_wake_up_reactor_again<Sut: Trigger>() {
        let (activator, observer) = Sut::Builder::new().create().unwrap();
        let reactor = create_reactor();
        let _guard = reactor.attach(&observer).unwrap();

        for _ in 0..NUMBER_OF_ACTIVATIONS {
            assert_that!(activator.activate(), is_ok);
        }
        assert_that!(reactor.try_wait(|_| {}), eq Ok(1));
        assert_that!(observer.reset(), eq Ok(true));

        assert_that!(reactor.try_wait(|_| {}), eq Ok(0));
    }

    #[conformance_test]
    pub fn activation_from_another_thread_wakes_up_blocked_reactor<Sut: Trigger>() {
        let _watchdog = Watchdog::new();
        let (activator, observer) = Sut::Builder::new().create().unwrap();
        let reactor = create_reactor();
        let _guard = reactor.attach(&observer).unwrap();

        thread_scope(|s| {
            s.thread_builder()
                .spawn(|| {
                    assert_that!(activator.activate(), is_ok);
                })
                .expect("failed to spawn thread");

            let result = reactor.blocking_wait(|_| {});
            assert_that!(result, eq Ok(1));
            assert_that!(observer.reset(), eq Ok(true));

            Ok(())
        })
        .expect("failed to spawn thread");
    }

    #[conformance_test]
    pub fn concurrent_activations_of_one_activator_succeed<Sut: Trigger>() {
        let _watchdog = Watchdog::new();
        const NUMBER_OF_THREADS: usize = 4;
        let (activator, observer) = Sut::Builder::new().create().unwrap();

        thread_scope(|s| {
            for _ in 0..NUMBER_OF_THREADS {
                s.thread_builder()
                    .spawn(|| {
                        for _ in 0..NUMBER_OF_ACTIVATIONS {
                            assert_that!(activator.activate(), is_ok);
                        }
                    })
                    .expect("failed to spawn thread");
            }

            Ok(())
        })
        .expect("failed to spawn thread");

        assert_that!(observer.reset(), eq Ok(true));
        assert_that!(observer.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn concurrent_activations_and_resets_succeed<Sut: Trigger>() {
        let _watchdog = Watchdog::new();
        const NUMBER_OF_THREADS: usize = 2;
        let (activator, observer) = Sut::Builder::new().create().unwrap();

        thread_scope(|s| {
            for _ in 0..NUMBER_OF_THREADS {
                s.thread_builder()
                    .spawn(|| {
                        for _ in 0..NUMBER_OF_ACTIVATIONS {
                            assert_that!(activator.activate(), is_ok);
                        }
                    })
                    .expect("failed to spawn thread");
                s.thread_builder()
                    .spawn(|| {
                        for _ in 0..NUMBER_OF_ACTIVATIONS {
                            assert_that!(observer.reset(), is_ok);
                        }
                    })
                    .expect("failed to spawn thread");
            }

            Ok(())
        })
        .expect("failed to spawn thread");

        assert_that!(observer.reset(), is_ok);
        assert_that!(observer.reset(), eq Ok(false));
    }

    #[conformance_test]
    pub fn error_display_works<Sut: Trigger>() {
        assert_that!(
            format!("{}", TriggerCreateError::InsufficientResources), eq "TriggerCreateError::InsufficientResources");
        assert_that!(
            format!("{}", TriggerCreateError::InternalError), eq "TriggerCreateError::InternalError");
        assert_that!(
            format!("{}", TriggerActivateError::Disconnected), eq "TriggerActivateError::Disconnected");
        assert_that!(
            format!("{}", TriggerActivateError::InternalError), eq "TriggerActivateError::InternalError");
        assert_that!(
            format!("{}", TriggerResetError::InternalError), eq "TriggerResetError::InternalError");
    }
}
