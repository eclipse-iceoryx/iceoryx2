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

use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;

pub trait Testing {
    fn sync(_id: String, _timeout: Duration) -> bool {
        true
    }

    /// Polls `f` with an adaptive backoff until it succeeds or `timeout`
    /// elapses. The distinct failure reasons observed are listed in the error.
    fn retry<F>(mut f: F, timeout: Duration) -> Result<(), String>
    where
        F: FnMut() -> Result<(), &'static str>,
    {
        let mut errors = BTreeSet::<&'static str>::new();

        let mut adaptive_wait = AdaptiveWaitBuilder::new()
            .create()
            .expect("failed to create adaptive wait");

        let succeeded = adaptive_wait
            .timed_wait_while(
                || -> Result<bool, ()> {
                    match f() {
                        Ok(()) => Ok(false),
                        Err(failure) => {
                            errors.insert(failure);
                            Ok(true)
                        }
                    }
                },
                timeout,
            )
            .expect("failed to wait");

        if succeeded {
            return Ok(());
        }

        errors.insert("Timeout exceeded.");
        let errors_formatted = errors
            .iter()
            .map(|e| format!("  - {}", e))
            .collect::<Vec<_>>()
            .join("\n");
        Err(errors_formatted)
    }
}
