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

#[generic_tests::define]
mod signal_mechanism {
    use core::{sync::atomic::AtomicU64, time::Duration};
    use std::sync::Barrier;

    use iceoryx2_bb_posix::clock::Time;
    use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};
    use iceoryx2_cal::event::signal_mechanism::{semaphore::Semaphore, SignalMechanism};
    const TIMEOUT: Duration = Duration::from_millis(25);

    #[test]
    fn notified_signal_does_not_block<Sut: SignalMechanism>() {
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

    #[test]
    fn try_wait_does_not_block_works<Sut: SignalMechanism>() {
        let mut sut = Sut::new();
        unsafe {
            assert_that!(sut.init(), is_ok);

            assert_that!(sut.try_wait(), eq Ok(false));
            assert_that!(sut.notify(), is_ok);
            assert_that!(sut.try_wait(), eq Ok(true));
            assert_that!(sut.try_wait(), eq Ok(false));
        }
    }

    fn wait_blocks<Sut: SignalMechanism, F: FnOnce(&Sut) -> bool + Send>(wait_call: F) {
        let _watchdog = Watchdog::new();
        let mut sut = Sut::new();
        let barrier = Barrier::new(2);
        let counter = AtomicU64::new(0);

        unsafe {
            assert_that!(sut.init(), is_ok);

            std::thread::scope(|s| {
                let t = s.spawn(|| {
                    barrier.wait();
                    assert_that!(wait_call(&sut), eq true);
                    counter.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                });

                barrier.wait();
                std::thread::sleep(TIMEOUT);
                assert_that!(counter.load(core::sync::atomic::Ordering::Relaxed), eq 0);
                sut.notify().unwrap();
                t.join().unwrap();

                assert_that!(counter.load(core::sync::atomic::Ordering::Relaxed), eq 1);
            });
        }
    }

    #[test]
    fn timed_wait_blocks<Sut: SignalMechanism>() {
        wait_blocks(|sut: &Sut| unsafe { sut.timed_wait(Duration::from_secs(999)).unwrap() });
    }

    #[test]
    fn blocking_wait_blocks<Sut: SignalMechanism>() {
        wait_blocks(|sut: &Sut| unsafe {
            sut.blocking_wait().unwrap();
            true
        });
    }

    #[test]
    fn timed_wait_blocks_at_least_for_timeout<Sut: SignalMechanism>() {
        let _watchdog = Watchdog::new();
        let mut sut = Sut::new();
        unsafe {
            assert_that!(sut.init(), is_ok);

            let now = Time::now().unwrap();
            assert_that!(sut.timed_wait(TIMEOUT), eq Ok(false));
            assert_that!(now.elapsed().unwrap(), time_at_least TIMEOUT);
        }
    }

    #[instantiate_tests(<Semaphore>)]
    mod semaphore {}
}
