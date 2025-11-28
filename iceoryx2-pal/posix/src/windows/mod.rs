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

pub use crate::os::posix::constants::*;
pub use crate::os::posix::dirent::*;
pub use crate::os::posix::errno::*;
pub use crate::os::posix::fcntl::*;
pub use crate::os::posix::mman::*;
pub use crate::os::posix::pthread::*;
pub use crate::os::posix::pwd::*;
pub use crate::os::posix::resource::*;
pub use crate::os::posix::sched::*;
pub use crate::os::posix::select::*;
pub use crate::os::posix::semaphore::*;
pub use crate::os::posix::signal::*;
pub use crate::os::posix::socket::*;
pub use crate::os::posix::stat::*;
pub use crate::os::posix::stdio::*;
pub use crate::os::posix::stdlib::*;
pub use crate::os::posix::string::*;
pub use crate::os::posix::support::*;
pub use crate::os::posix::time::*;
pub use crate::os::posix::types::*;
pub use crate::os::posix::unistd::*;
