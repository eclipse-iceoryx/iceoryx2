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

use core::time::Duration;

use iceoryx2_bb_testing::test_fail;

pub trait Testing {
    fn sync(_id: String, _timeout: Duration) -> bool {
        true
    }

    fn retry<F>(mut f: F, period: Duration, max_attempts: Option<usize>)
    where
        F: FnMut() -> Result<(), &'static str>,
    {
        let mut attempt = 0;

        loop {
            match f() {
                Ok(_) => return,
                Err(failure) => {
                    if let Some(max_attempts) = max_attempts {
                        if attempt >= max_attempts {
                            test_fail!("{} after {} attempts", failure, attempt);
                        }
                    }
                }
            }

            std::thread::sleep(period);
            attempt += 1;
        }
    }
}
