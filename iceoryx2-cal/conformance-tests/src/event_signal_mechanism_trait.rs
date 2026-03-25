// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_conformance_test_macros::conformance_test_module;

#[allow(clippy::module_inception)]
#[conformance_test_module]
pub mod event_signal_mechanism_trait {
    use core::time::Duration;
    use iceoryx2_bb_concurrency::atomic::AtomicU64;
    use iceoryx2_bb_conformance_test_macros::conformance_test;
    use iceoryx2_bb_posix::clock::nanosleep;
    use iceoryx2_bb_posix::ipc_capable::Handle;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_posix::{
        barrier::{BarrierBuilder, BarrierHandle},
        clock::Time,
    };
    use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};
    use iceoryx2_cal::event::signal_mechanism::SignalMechanism;

    const TIMEOUT: Duration = Duration::from_millis(25);

    #[conformance_test]
    pub fn notified_signal_does_not_block<Sut: SignalMechanism>() {
        let _watchdog = Watchdog::new();
        let mut sut = Sut::new();
        unsafe {
            assert_that!(sut.init(), is_ok);

            assert_that!(sut.notify(), is_ok);
            assert_that!(sut.try_wait(), eq Ok(true));

            assert_that!(sut.notify(), is_ok);
            assert_that!(sut.timed_wait(TIMEOUT), eq Ok(true));

            assert_that!(sut.notify(), is_ok);
            assert_that!(sut.blocking_wait(), is_ok);
        }
    }

    #[conformance_test]
    pub fn try_wait_does_not_block_works<Sut: SignalMechanism>() {
        let mut sut = Sut::new();
        unsafe {
            assert_that!(sut.init(), is_ok);

            assert_that!(sut.try_wait(), eq Ok(false));
            assert_that!(sut.notify(), is_ok);
            assert_that!(sut.try_wait(), eq Ok(true));
            assert_that!(sut.try_wait(), eq Ok(false));
        }
    }

    pub fn wait_blocks<Sut: SignalMechanism, F: FnOnce(&Sut) -> bool + Send>(wait_call: F) {
        let _watchdog = Watchdog::new();
        let mut sut = Sut::new();
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
        let counter = AtomicU64::new(0);

        unsafe {
            assert_that!(sut.init(), is_ok);

            thread_scope(|s| {
                s.thread_builder().spawn(|| {
                    barrier.wait();
                    assert_that!(wait_call(&sut), eq true);
                    counter.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                })?;

                barrier.wait();
                nanosleep(TIMEOUT).unwrap();
                assert_that!(counter.load(core::sync::atomic::Ordering::Relaxed), eq 0);
                sut.notify().unwrap();

                Ok(())
            })
            .unwrap();

            assert_that!(counter.load(core::sync::atomic::Ordering::Relaxed), eq 1);
        }
    }

    #[conformance_test]
    pub fn timed_wait_blocks<Sut: SignalMechanism>() {
        wait_blocks(|sut: &Sut| unsafe { sut.timed_wait(Duration::from_secs(999)).unwrap() });
    }

    #[conformance_test]
    pub fn blocking_wait_blocks<Sut: SignalMechanism>() {
        wait_blocks(|sut: &Sut| unsafe {
            sut.blocking_wait().unwrap();
            true
        });
    }

    #[conformance_test]
    pub fn timed_wait_blocks_at_least_for_timeout<Sut: SignalMechanism>() {
        let _watchdog = Watchdog::new();
        let mut sut = Sut::new();
        unsafe {
            assert_that!(sut.init(), is_ok);

            let now = Time::now().unwrap();
            assert_that!(sut.timed_wait(TIMEOUT), eq Ok(false));
            assert_that!(now.elapsed().unwrap(), time_at_least TIMEOUT);
        }
    }
}
