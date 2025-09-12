// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use core::fmt::Display;

/// Represents the crates version acquired through the internal environment variables set by cargo,
/// ("CARGO_PKG_VERSION_{MAJOR|MINOR|PATCH}").
///
/// # Example
///
/// ```
/// use iceoryx2_bb_elementary::package_version::PackageVersion;
///
/// let version = PackageVersion::get();
///
/// println!("package version: {}", version);
/// println!(" major: {}", version.major());
/// println!(" minor: {}", version.minor());
/// println!(" patch: {}", version.patch());
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PackageVersion(u64);

impl PackageVersion {
    /// Creates a [`PackageVersion`] from a raw encoded u64
    pub fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Converts the [`PackageVersion`] to an u64
    pub fn to_u64(&self) -> u64 {
        self.0
    }

    fn from_version(major: u16, minor: u16, patch: u16) -> Self {
        Self(((major as u64) << 32) | ((minor as u64) << 16) | patch as u64)
    }

    /// Returns the major part of the version
    pub fn major(&self) -> u16 {
        ((self.0 >> 32) & (u16::MAX as u64)) as u16
    }

    /// Returns the minor part of the version
    pub fn minor(&self) -> u16 {
        ((self.0 >> 16) & (u16::MAX as u64)) as u16
    }

    /// Returns the patch part of the version
    pub fn patch(&self) -> u16 {
        ((self.0) & (u16::MAX as u64)) as u16
    }

    /// Returns the current [`PackageVersion`]
    pub fn get() -> PackageVersion {
        const MAJOR: u16 = 0;
        const MINOR: u16 = 7;
        const PATCH: u16 = 0;

        PackageVersion::from_version(MAJOR, MINOR, PATCH)
    }

    /// Returns the version as a str using get internally
    pub fn get_str() -> &'static str {
        // Build a string from the version using the Display implementation
        let version = PackageVersion::get();
        let version_str = format!("{version}");
        Box::leak(version_str.into_boxed_str())
    }
}

impl Display for PackageVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}.{}.{}", self.major(), self.minor(), self.patch())
    }
}
