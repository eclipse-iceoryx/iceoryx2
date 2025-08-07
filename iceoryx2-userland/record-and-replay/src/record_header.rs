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

use iceoryx2::prelude::MessagingPattern;
use iceoryx2_bb_elementary::package_version::PackageVersion;

use crate::recorder::ServiceTypes;

#[repr(C)]
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
/// Represents a semver version
pub struct Version {
    /// Major version part
    pub major: u16,
    /// Minor version part
    pub minor: u16,
    /// Patch version part
    pub patch: u16,
}

impl From<PackageVersion> for Version {
    fn from(value: PackageVersion) -> Self {
        Version {
            major: value.major(),
            minor: value.minor(),
            patch: value.patch(),
        }
    }
}

#[repr(C)]
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
/// Represents the header of a recorded file which identifies the type details and iceoryx2
/// version used when the data was captured.
pub struct RecordHeader {
    /// The version of iceoryx2 used when the data was captured.
    pub version: Version,
    /// The types to which the stored payload corresponds.
    pub types: ServiceTypes,
    /// The messaging pattern of the recorded service.
    pub messaging_pattern: MessagingPattern,
}
