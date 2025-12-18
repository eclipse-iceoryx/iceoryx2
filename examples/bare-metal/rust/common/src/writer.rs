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

#[cfg(feature = "semihosting")]
use crate::semihosting;

use iceoryx2_bb_concurrency::spin_lock::SpinLock;

pub static OUTPUT: SpinLock<OutputWriter> = SpinLock::new(OutputWriter);

pub struct OutputWriter;

impl core::fmt::Write for OutputWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        #[cfg(feature = "semihosting")]
        semihosting::write0(s);

        #[cfg(not(feature = "semihosting"))]
        {
            // No-op when semihosting is disabled
            // Add UART output or other logging mechanism here
            let _ = s;
        }
        Ok(())
    }
}
