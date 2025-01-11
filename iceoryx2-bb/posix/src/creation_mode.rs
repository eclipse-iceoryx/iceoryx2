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

//! The [`CreationMode`] describes how certain posix resources should be created.

use core::fmt::Display;
use iceoryx2_pal_posix::*;

/// Describes how new resources like [`crate::file::File`], [`crate::shared_memory::SharedMemory`]
/// or others should be created.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Default)]
pub enum CreationMode {
    /// Create resource, if its already existing fail.
    #[default]
    CreateExclusive,
    /// Always remove existing resource and override it with new one
    PurgeAndCreate,
    /// Either open the new resource or create it when it is not existing
    OpenOrCreate,
}

impl CreationMode {
    pub fn as_oflag(&self) -> posix::int {
        match self {
            CreationMode::PurgeAndCreate => posix::O_CREAT | posix::O_EXCL,
            CreationMode::CreateExclusive => posix::O_CREAT | posix::O_EXCL,
            CreationMode::OpenOrCreate => posix::O_CREAT,
        }
    }
}

impl Display for CreationMode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "CreationMode::{}",
            match self {
                CreationMode::CreateExclusive => "CreateExclusive",
                CreationMode::PurgeAndCreate => "PurgeAndCreate",
                CreationMode::OpenOrCreate => "OpenOrCreate",
            }
        )
    }
}
