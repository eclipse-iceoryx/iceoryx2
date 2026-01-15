// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

pub mod writer;

pub fn is_terminal() -> bool {
    #[cfg(feature = "std")]
    {
        use std::io::IsTerminal;
        std::io::stderr().is_terminal()
    }

    #[cfg(not(feature = "std"))]
    false
}

#[macro_export]
macro_rules! cout {
    ($($arg:tt)*) => {{
        use core::fmt::Write as _;
        let _ = core::writeln!($crate::writer::stdout(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! cerr {
    ($($arg:tt)*) => {
        use core::fmt::Write as _;
        let _ = core::writeln!($crate::writer::stderr(), $($arg)*);
    };
}
