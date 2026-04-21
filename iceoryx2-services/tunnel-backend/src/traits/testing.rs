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

use alloc::collections::btree_set::BTreeSet;
use alloc::vec::Vec;
use alloc::{format, string::String};

use iceoryx2_bb_posix::clock::nanosleep;

pub trait Testing {
    fn sync(_id: String, _timeout: Duration) -> bool {
        true
    }

    fn retry<F>(mut f: F, period: Duration, max_attempts: Option<usize>) -> Result<(), String>
    where
        F: FnMut() -> Result<(), &'static str>,
    {
        let mut attempt = 0;

        let mut errors = BTreeSet::<&'static str>::new();

        loop {
            match f() {
                Ok(_) => return Ok(()),
                Err(failure) => {
                    errors.insert(failure);
                    if let Some(max_attempts) = max_attempts {
                        if attempt >= max_attempts {
                            errors.insert("Retry attempts exceeded.");

                            let errors_formatted = errors
                                .iter()
                                .map(|e| format!("  - {}", e))
                                .collect::<Vec<_>>()
                                .join("\n");
                            return Err(errors_formatted);
                        }
                    }
                }
            }

            nanosleep(period).unwrap();
            attempt += 1;
        }
    }
}
