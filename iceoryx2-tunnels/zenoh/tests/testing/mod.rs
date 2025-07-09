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

use std::time::Duration;

use iceoryx2_bb_testing::test_fail;

use zenoh::Wait;

/// Repeatedly attempts to execute a function until it succeeds or reaches the maximum number of attempts.
///
/// Required for operations that involve zenoh as the background thread makes the
/// execution indeterministic.
///
/// # Arguments
///
/// * `f` - A function that returns `Result<(), &'static str>`. The function is considered successful when it returns `Ok(())`.
/// * `period` - The duration to wait between retry attempts.
/// * `max_attempts` - An optional maximum number of retry attempts. If `None`, the function will retry indefinitely.
///
/// # Behavior
///
/// If the function succeeds (returns `Ok(())`), this function returns immediately.
/// If the function fails and `max_attempts` is reached, this function will call `test_fail!` with the error message.
/// Otherwise, it will sleep for the specified period and try again.
pub fn retry<F>(mut f: F, period: Duration, max_attempts: Option<usize>)
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
                        test_fail!("{}, after {} attempts", failure, attempt);
                    }
                }
            }
        }

        std::thread::sleep(period);
        attempt += 1;
    }
}

/// Waits for a Zenoh match on the specified key for up to the specified duration.
///
/// This function can be used to stall execution of test logic until the zenoh background
/// thread has woken up to set up matches. The assumption is that if this unrelated subscriber
/// is matched, other matches on this key should also have been processed.
///
/// # Arguments
///
/// * `z_key` - The Zenoh key to subscribe to
/// * `timeout` - Maximum duration to wait for a match
///
/// # Returns
///
/// * `true` if a match was found within the timeout period
/// * `false` if the timeout was reached without finding a match
pub fn wait_for_zenoh_match(z_key: String, timeout: Duration) -> bool {
    let start_time = std::time::Instant::now();
    let z_config = zenoh::Config::default();
    let z_session = zenoh::open(z_config.clone()).wait().unwrap();
    let z_subscriber = z_session.declare_subscriber(z_key).wait().unwrap();

    while z_subscriber.sender_count() == 0 {
        if start_time.elapsed() >= timeout {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    true
}
