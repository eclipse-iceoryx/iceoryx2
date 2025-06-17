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

//! [`Metadata`] contains all informations like type, credentials, size, access times about every
//! structure which has a file handle representation. Every struct which implements the
//! [`crate::file_descriptor::FileDescriptorManagement`] trait can emit a [`Metadata`].
//! One struct is for instance [`crate::file::File`].

use crate::clock::{ClockType, Time, TimeBuilder};
use crate::file_type::FileType;
use crate::group::Gid;
use crate::permission::{Permission, PermissionExt};
use crate::user::Uid;
use iceoryx2_pal_posix::*;

/// Contains all informations like type, credentials, size, access times about every
/// structure which has a file handle representation. Every struct which implements the
/// [`crate::file_descriptor::FileDescriptorManagement`] trait can emit a [`Metadata`].
/// One struct is for instance [`crate::file::File`].
#[derive(Debug)]
pub struct Metadata {
    file_type: FileType,
    uid: Uid,
    gid: Gid,
    size: u64,
    block_size: u64,
    permission: Permission,
    access_time: Time,
    modification_time: Time,
    creation_time: Time,
    device_id: u64,
}

impl Metadata {
    pub fn access_time(&self) -> Time {
        self.access_time
    }

    pub fn creation_time(&self) -> Time {
        self.creation_time
    }

    /// returns the size of the file
    pub fn size(&self) -> u64 {
        self.size
    }

    /// returns the block size of the file. the size which the file occupies on the file system.
    pub fn block_size(&self) -> u64 {
        self.block_size
    }

    pub fn device_id(&self) -> u64 {
        self.device_id
    }

    pub fn modification_time(&self) -> Time {
        self.modification_time
    }

    /// the access permissions of the file for owner, group and other
    pub fn permission(&self) -> Permission {
        self.permission
    }

    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// returns the user id (uid) of the files owner
    pub fn uid(&self) -> Uid {
        self.uid
    }

    /// returns the group id (gid) of the files owner
    pub fn gid(&self) -> Gid {
        self.gid
    }

    pub(crate) fn create(attr: &posix::stat_t) -> Metadata {
        Self {
            access_time: TimeBuilder::new()
                .clock_type(ClockType::Realtime)
                .seconds(attr.st_atime as u64)
                .create(),
            creation_time: TimeBuilder::new()
                .clock_type(ClockType::Realtime)
                .seconds(attr.st_ctime as u64)
                .create(),
            size: attr.st_size as u64,
            block_size: attr.st_size as u64,
            device_id: attr.st_rdev as _,
            modification_time: TimeBuilder::new()
                .clock_type(ClockType::Realtime)
                .seconds(attr.st_mtime as u64)
                .create(),
            permission: attr.st_mode.as_permission(),
            file_type: FileType::from_mode_t(attr.st_mode),
            uid: Uid::new_from_native(attr.st_uid),
            gid: Gid::new_from_native(attr.st_gid),
        }
    }
}
