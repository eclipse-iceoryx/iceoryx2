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

use crate::IsTerminal;

/// Zero-sized stateless handle to STDOUT.
pub struct Stdout;

/// Zero-sized stateless handle to STDERR.
pub struct Stderr;

/// Convenience function to retrieve a handle to stdout.
pub fn stdout() -> Stdout {
    Stdout
}

/// Convenience function to retrieve a handle to stderr.
pub fn stderr() -> Stderr {
    Stderr
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
impl IsTerminal for Stdout {
    fn is_terminal(&self) -> bool {
        use std::io::IsTerminal;
        std::io::stdout().is_terminal()
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

#[cfg(feature = "std")]
impl IsTerminal for Stderr {
    fn is_terminal(&self) -> bool {
        use std::io::IsTerminal;
        std::io::stderr().is_terminal()
    }
}

#[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "nto",)))]
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

#[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "nto",)))]
impl IsTerminal for Stdout {
    fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "nto",)))]
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

#[cfg(all(not(feature = "std"), any(target_os = "linux", target_os = "nto",)))]
impl IsTerminal for Stderr {
    fn is_terminal(&self) -> bool {
        true
    }
}

#[cfg(all(
    not(feature = "std"),
    not(any(target_os = "linux", target_os = "nto",))
))]
impl Write for Stdout {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        Ok(())
    }
}

#[cfg(all(
    not(feature = "std"),
    not(any(target_os = "linux", target_os = "nto",))
))]
impl IsTerminal for Stdout {
    fn is_terminal(&self) -> bool {
        false
    }
}

#[cfg(all(
    not(feature = "std"),
    not(any(target_os = "linux", target_os = "nto",))
))]
impl Write for Stderr {
    fn write_str(&mut self, _s: &str) -> fmt::Result {
        Ok(())
    }
}

#[cfg(all(
    not(feature = "std"),
    not(any(target_os = "linux", target_os = "nto",))
))]
impl IsTerminal for Stderr {
    fn is_terminal(&self) -> bool {
        false
    }
}
