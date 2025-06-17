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

//! Represents the [`Ownership`] in a unix environment consisting of user and group. Can be used in
//! combination with [`crate::file_descriptor::FileDescriptorManagement`] to set the
//! credentials of [`crate::file::File`], [`crate::shared_memory::SharedMemory`] and others.
//! # Example
//!
//! ```rust,ignore
//! use iceoryx2_bb_posix::ownership::*;
//! use iceoryx2_bb_posix::user::UserExt;
//! use iceoryx2_bb_posix::group::GroupExt;
//!
//! let ownership = OwnershipBuilder::new().uid("root".as_user().expect("no such user").uid())
//!                                        .gid("root".as_group().expect("no such group").gid())
//!                                        .create();
//!
//! println!("The uid/gid of root/root is: {}/{}", ownership.uid(), ownership.gid());
//! ```

use crate::group::Gid;
use crate::user::Uid;

/// Defines the owner in a unix environment consisting of user and group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ownership {
    uid: Uid,
    gid: Gid,
}

/// The builder to the [`Ownership`] struct.
/// One can use [`crate::user::User`] and [`crate::group::Group`] to acquire the ids quickly from
/// the names.
pub struct OwnershipBuilder {}

impl Default for OwnershipBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OwnershipBuilderWithUid {
    uid: Uid,
}

pub struct OwnershipBuilderWithUidAndGid {
    ownership: Ownership,
}

impl OwnershipBuilder {
    pub fn new() -> Self {
        Self {}
    }

    /// Sets the user id
    pub fn uid(self, uid: Uid) -> OwnershipBuilderWithUid {
        OwnershipBuilderWithUid { uid }
    }
}

impl OwnershipBuilderWithUid {
    /// Sets the group id
    pub fn gid(self, gid: Gid) -> OwnershipBuilderWithUidAndGid {
        OwnershipBuilderWithUidAndGid {
            ownership: Ownership { uid: self.uid, gid },
        }
    }
}

impl OwnershipBuilderWithUidAndGid {
    pub fn create(self) -> Ownership {
        self.ownership
    }
}

impl Ownership {
    /// returns the user id
    pub fn uid(&self) -> Uid {
        self.uid
    }

    /// returns the group id
    pub fn gid(&self) -> Gid {
        self.gid
    }
}
