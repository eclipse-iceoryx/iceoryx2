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

pub mod constants;
pub mod dirent;
pub mod errno;
pub mod fcntl;
pub mod mman;
pub mod pthread;
pub mod pwd;
pub mod resource;
pub mod sched;
pub mod select;
pub mod semaphore;
#[doc(hidden)]
pub mod settings;
pub mod signal;
pub mod socket;
pub mod stat;
pub mod stdio;
pub mod stdlib;
pub mod string;
pub mod support;
pub mod time;
pub mod types;
pub mod unistd;
#[macro_use]
mod win32_call;
pub mod win32_handle_translator;
pub mod win32_security_attributes;
mod win32_udp_port_to_uds_name;

pub use crate::windows::constants::*;
pub use crate::windows::dirent::*;
pub use crate::windows::errno::*;
pub use crate::windows::fcntl::*;
pub use crate::windows::mman::*;
pub use crate::windows::pthread::*;
pub use crate::windows::pwd::*;
pub use crate::windows::resource::*;
pub use crate::windows::sched::*;
pub use crate::windows::select::*;
pub use crate::windows::semaphore::*;
pub use crate::windows::signal::*;
pub use crate::windows::socket::*;
pub use crate::windows::stat::*;
pub use crate::windows::stdio::*;
pub use crate::windows::stdlib::*;
pub use crate::windows::string::*;
pub use crate::windows::support::*;
pub use crate::windows::time::*;
pub use crate::windows::types::*;
pub use crate::windows::unistd::*;
