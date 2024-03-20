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
    use std::time::Duration;

    use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};
    use iceoryx2_cal::event::signal_mechanism::{semaphore::Semaphore, SignalMechanism};
    const TIMEOUT: Duration = Duration::from_millis(25);

    #[test]
    fn try_wait_works<Sut: SignalMechanism>() {
        let mut sut = Sut::new();
        assert_that!(sut.init(), is_ok);

        assert_that!(sut.try_wait(), eq Ok(false));
        //assert_that!(sut.notify(), is_ok);
        //assert_that!(sut.try_wait(), eq Ok(true));
        //assert_that!(sut.try_wait(), eq Ok(false));
    }

    #[test]
    fn notified_signal_does_not_block<Sut: SignalMechanism>() {
        let _watchdog = Watchdog::new(Duration::from_secs(1));
        let mut sut = Sut::new();
        assert_that!(sut.init(), is_ok);

        assert_that!(sut.notify(), is_ok);
        assert_that!(sut.try_wait(), eq Ok(true));

        assert_that!(sut.notify(), is_ok);
        assert_that!(sut.timed_wait(TIMEOUT), eq Ok(true));

        assert_that!(sut.notify(), is_ok);
        assert_that!(sut.blocking_wait(), is_ok);
    }

    #[instantiate_tests(<Semaphore>)]
    mod bitset {}
}
