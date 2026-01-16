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

pub fn stdout() -> &'static mut Stdout {
    static mut STDOUT: Stdout = Stdout;

    unsafe {
        #[allow(static_mut_refs)]
        &mut STDOUT
    }
}

pub fn stderr() -> &'static mut Stderr {
    static mut STDERR: Stderr = Stderr;

    unsafe {
        #[allow(static_mut_refs)]
        &mut STDERR
    }
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
        use iceoryx2_pal_posix::*;

        let result = unsafe {
            posix::write(
                posix::STDOUT_FILENO as _,
                s.as_ptr() as *const posix::void,
                s.len() as posix::size_t,
            )
        };

        if result < 0 {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "posix")]
impl Write for Stderr {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use iceoryx2_pal_posix::*;

        let result = unsafe {
            posix::write(
                posix::STDERR_FILENO as _,
                s.as_ptr() as *const posix::void,
                s.len() as posix::size_t,
            )
        };

        if result < 0 {
            Err(core::fmt::Error)
        } else {
            Ok(())
        }
    }
}

#[cfg(not(any(feature = "std", feature = "posix")))]
impl Write for Stdout {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        Ok(())
    }
}

#[cfg(not(any(feature = "std", feature = "posix")))]
impl Write for Stderr {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        Ok(())
    }
}
