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

use core::fmt::{self, Write};

pub struct Stdout;
pub struct Stderr;

static mut STDOUT: Stdout = Stdout;
static mut STDERR: Stderr = Stderr;

pub fn stdout() -> &'static mut Stdout {
    unsafe { &mut *core::ptr::addr_of_mut!(STDOUT) }
}

pub fn stderr() -> &'static mut Stderr {
    unsafe { &mut *core::ptr::addr_of_mut!(STDERR) }
}

#[cfg(feature = "std")]
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use std::io::Write as IoWrite;
        std::io::stdout()
            .write_all(s.as_bytes())
            .map_err(|_| fmt::Error)
    }
}

#[cfg(feature = "std")]
impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use std::io::Write as IoWrite;
        std::io::stderr()
            .write_all(s.as_bytes())
            .map_err(|_| fmt::Error)
    }
}

#[cfg(feature = "posix")]
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(())
    }
}

#[cfg(feature = "posix")]
impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Ok(())
    }
}
