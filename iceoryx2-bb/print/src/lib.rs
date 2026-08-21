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

pub use iceoryx2_pal_print::IsTerminal;

// keep this in sync with iceoryx2-pal/print/lib.rs
pub use iceoryx2_pal_print::RESET;

// Styles
pub use iceoryx2_pal_print::BOLD;
pub use iceoryx2_pal_print::DIM;
pub use iceoryx2_pal_print::ITALIC;
pub use iceoryx2_pal_print::UNDERLINE;

// Standard colors
pub use iceoryx2_pal_print::BLUE;
pub use iceoryx2_pal_print::GREEN;
pub use iceoryx2_pal_print::RED;
pub use iceoryx2_pal_print::WHITE;
pub use iceoryx2_pal_print::YELLOW;

// Bright colors
pub use iceoryx2_pal_print::BRIGHT_BLUE;
pub use iceoryx2_pal_print::BRIGHT_GREEN;
pub use iceoryx2_pal_print::BRIGHT_RED;
pub use iceoryx2_pal_print::BRIGHT_WHITE;
pub use iceoryx2_pal_print::BRIGHT_YELLOW;

pub use iceoryx2_pal_print::stderr;
pub use iceoryx2_pal_print::stdout;

pub use iceoryx2_pal_print::cerr;
pub use iceoryx2_pal_print::cerrln;
pub use iceoryx2_pal_print::cout;
pub use iceoryx2_pal_print::coutln;
