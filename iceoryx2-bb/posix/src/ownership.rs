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

/// Defines the owner in a unix environment consisting of user and group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ownership {
    uid: u32,
    gid: u32,
}

/// The builder to the [`Ownership`] struct.
/// One can use [`crate::user::User`] and [`crate::group::Group`] to acquire the ids quickly from
/// the names.
pub struct OwnershipBuilder {
    ownership: Ownership,
}

impl Default for OwnershipBuilder {
    fn default() -> Self {
        OwnershipBuilder {
            ownership: Ownership {
                uid: u32::MAX,
                gid: u32::MAX,
            },
        }
    }
}

impl OwnershipBuilder {
    pub fn new() -> OwnershipBuilder {
        Self::default()
    }

    /// Sets the user id
    pub fn uid(mut self, uid: u32) -> Self {
        self.ownership.uid = uid;
        self
    }

    /// Sets the group id
    pub fn gid(mut self, gid: u32) -> Self {
        self.ownership.gid = gid;
        self
    }

    pub fn create(self) -> Ownership {
        self.ownership
    }
}

impl Ownership {
    /// returns the user id
    pub fn uid(&self) -> u32 {
        self.uid
    }

    /// returns the group id
    pub fn gid(&self) -> u32 {
        self.gid
    }
}
