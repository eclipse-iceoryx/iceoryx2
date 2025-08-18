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

use core::fmt::Display;
use core::ops::{BitOr, BitOrAssign, Not};
use iceoryx2_pal_posix::*;

type ModeType = posix::mode_t;

/// Defines the permission of a file or directory in a POSIX system.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permission(ModeType);

impl Permission {
    pub const OWNER_READ: Self = Self(0o0400);
    pub const OWNER_WRITE: Self = Self(0o0200);
    pub const OWNER_EXEC: Self = Self(0o0100);
    pub const OWNER_ALL: Self = Self(0o0700);

    pub const GROUP_READ: Self = Self(0o0040);
    pub const GROUP_WRITE: Self = Self(0o0020);
    pub const GROUP_EXEC: Self = Self(0o0010);
    pub const GROUP_ALL: Self = Self(0o0070);

    pub const OTHERS_READ: Self = Self(0o0004);
    pub const OTHERS_WRITE: Self = Self(0o0002);
    pub const OTHERS_EXEC: Self = Self(0o0001);
    pub const OTHERS_ALL: Self = Self(0o0007);

    pub const ALL: Self = Self(0o0777);

    pub const SET_UID: Self = Self(0o4000);
    pub const SET_GID: Self = Self(0o2000);
    pub const STICKY_BIT: Self = Self(0o1000);

    pub const MASK: Self = Self(0o7777);
    pub const UNKNOWN: Self = Self(0xFFFF);

    pub fn none() -> Self {
        Self(0)
    }

    pub fn bits(&self) -> ModeType {
        self.0
    }
}

impl BitOrAssign for Permission {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitOr for Permission {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl Not for Permission {
    type Output = Self;

    fn not(self) -> Self::Output {
        Permission(!self.0)
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
        (self.0 & rhs.0) != 0
    }

    /// Converts the permissions into the C type mode_t
    pub fn as_mode(&self) -> posix::mode_t {
        self.0
    }
}

impl Display for Permission {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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

        write!(f, "{owner}{group}{others}")
    }
}
