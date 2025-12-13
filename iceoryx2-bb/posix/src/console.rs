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

use iceoryx2_pal_posix::*;

pub enum Stream {
    StandardOutput,
    StandardError,
}

impl Stream {
    fn as_file_descriptor(&self) -> posix::int {
        match self {
            Self::StandardOutput => posix::STDOUT_FILENO as posix::int,
            Self::StandardError => posix::STDERR_FILENO as posix::int,
        }
    }
}

pub fn write(stream: Stream, str: &str) -> core::fmt::Result {
    let result = unsafe {
        posix::write(
            stream.as_file_descriptor(),
            str.as_ptr() as *const posix::void,
            str.len() as posix::size_t,
        )
    };

    if result < 0 {
        Err(core::fmt::Error)
    } else {
        Ok(())
    }
}
