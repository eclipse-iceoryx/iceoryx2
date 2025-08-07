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

use iceoryx2::prelude::{MessagingPattern, ServiceName};
use iceoryx2_bb_elementary::package_version::PackageVersion;

use crate::recorder::ServiceTypes;

/// Defines the current file format version of the human readable format
pub const FILE_FORMAT_HUMAN_READABLE_VERSION: u64 = 1;

/// Defines the current file format version of the iox2dump version
pub const FILE_FORMAT_IOX2_DUMP_VERSION: u64 = 1;

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
/// Contains the version, message and type details
pub struct RecordHeaderDetails {
    /// Defines the file format version
    pub file_format_version: u64,
    /// The types to which the stored payload corresponds.
    pub types: ServiceTypes,
    /// The messaging pattern of the recorded service.
    pub messaging_pattern: MessagingPattern,
}

#[repr(C)]
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
/// Represents the header of a recorded file which identifies the type details and iceoryx2
/// version used when the data was captured.
pub struct RecordHeader {
    /// The version of iceoryx2 used when the data was captured.
    pub iceoryx2_version: Version,
    /// The name of the service that was recorded.
    pub service_name: ServiceName,
    /// The version, message and type details
    pub details: RecordHeaderDetails,
}
