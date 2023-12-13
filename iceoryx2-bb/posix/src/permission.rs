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

//! Defines the [`Permission`] of a file or directory in a POSIX system.
//! Can be used in
//! combination with [`crate::file_descriptor::FileDescriptorManagement`] to set the
//! credentials of [`crate::file::File`], [`crate::shared_memory::SharedMemory`] and others.

use bitflags::bitflags;
use iceoryx2_pal_posix::*;
use std::fmt::Display;

type ModeType = posix::mode_t;

bitflags! {
    /// Defines the permission of a file or directory in a POSIX system.
   #[derive(Default)]
    pub struct Permission: ModeType {
        const OWNER_READ = 0o0400;
        const OWNER_WRITE = 0o0200;
        const OWNER_EXEC = 0o0100;
        const OWNER_ALL = 0o0700;

        const GROUP_READ = 0o0040;
        const GROUP_WRITE = 0o0020;
        const GROUP_EXEC = 0o0010;
        const GROUP_ALL = 0o0070;

        const OTHERS_READ = 0o0004;
        const OTHERS_WRITE = 0o0002;
        const OTHERS_EXEC = 0o0001;
        const OTHERS_ALL = 0o0007;

        const ALL = 0o0777;

        const SET_UID = 0o4000;
        const SET_GID= 0o2000;
        const STICKY_BIT = 0o1000;

        const MASK = 0o7777;
        const UNKNOWN = 0xFFFF;
    }
}

impl Permission {
    pub fn none() -> Self {
        Self { bits: 0 }
    }
}

/// Trait which allows other types like integers to be converted into [`Permission`].
pub trait PermissionExt {
    /// converts value into [`Permission`]
    fn as_permission(&self) -> Permission;
}

impl PermissionExt for posix::mode_t {
    fn as_permission(&self) -> Permission {
        let mut p = Permission::none();

        let owner = self & posix::S_IRWXU;
        if owner & posix::S_IRUSR != 0 {
            p |= Permission::OWNER_READ;
        }
        if owner & posix::S_IWUSR != 0 {
            p |= Permission::OWNER_WRITE;
        }
        if owner & posix::S_IXUSR != 0 {
            p |= Permission::OWNER_EXEC;
        }

        let group = self & posix::S_IRWXG;
        if group & posix::S_IRGRP != 0 {
            p |= Permission::GROUP_READ;
        }
        if group & posix::S_IWGRP != 0 {
            p |= Permission::GROUP_WRITE;
        }
        if group & posix::S_IXGRP != 0 {
            p |= Permission::GROUP_EXEC;
        }

        let others = self & posix::S_IRWXO;
        if others & posix::S_IROTH != 0 {
            p |= Permission::OTHERS_READ;
        }
        if others & posix::S_IWOTH != 0 {
            p |= Permission::OTHERS_WRITE;
        }
        if others & posix::S_IXOTH != 0 {
            p |= Permission::OTHERS_EXEC;
        }

        if self & posix::S_ISUID != 0 {
            p |= Permission::SET_UID;
        }
        if self & posix::S_ISGID != 0 {
            p |= Permission::SET_GID;
        }

        if self & posix::S_ISVTX != 0 {
            p |= Permission::STICKY_BIT;
        }

        p
    }
}

impl Permission {
    /// Returns true when self contains the permissions of the rhs, otherwise false.
    pub fn has(&self, rhs: Permission) -> bool {
        (*self & rhs) != Permission::none()
    }

    /// Converts the permissions into the C type mode_t
    pub fn as_mode(&self) -> posix::mode_t {
        self.bits
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut owner = "---".to_string();
        if self.has(Permission::OWNER_READ) {
            owner.replace_range(0..1, "r");
        }
        if self.has(Permission::OWNER_WRITE) {
            owner.replace_range(1..2, "w");
        }
        if self.has(Permission::OWNER_EXEC) {
            owner.replace_range(2..3, "x");
        }

        let mut group = "---".to_string();
        if self.has(Permission::GROUP_READ) {
            group.replace_range(0..1, "r");
        }
        if self.has(Permission::GROUP_WRITE) {
            group.replace_range(1..2, "w");
        }
        if self.has(Permission::GROUP_EXEC) {
            group.replace_range(2..3, "x");
        }

        let mut others = "---".to_string();
        if self.has(Permission::OTHERS_READ) {
            others.replace_range(0..1, "r")
        }
        if self.has(Permission::OTHERS_WRITE) {
            others.replace_range(1..2, "w")
        }
        if self.has(Permission::OTHERS_EXEC) {
            others.replace_range(2..3, "x")
        }

        let mut bits = String::new();
        if self.has(Permission::STICKY_BIT) {
            bits += "StickyBit, ";
        }
        if self.has(Permission::SET_UID) {
            bits += "SetUid, ";
        }
        if self.has(Permission::SET_GID) {
            bits += "SetGid, ";
        }

        if bits.is_empty() {
            bits += "-";
        } else {
            bits.truncate(bits.len() - 2);
        }

        write!(
            f,
            "Permission {{ Owner: {}, Group: {}, Others: {}, Bits: {} }}",
            owner, group, others, bits
        )
    }
}
