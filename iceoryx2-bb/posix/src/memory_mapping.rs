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

use crate::{file::AccessMode, file_descriptor::FileDescriptor};
use iceoryx2_bb_system_types::file_path::FilePath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingPermission {
    Read,
    Write,
    ReadWrite,
    Exec,
    ReadExec,
    WriteExec,
    ReadWriteExec,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingBehavior {
    /// Memory changes are written back and are visible to other processes
    Shared,
    /// Changes are local to the process.
    Private,
}

pub struct MemoryMappingBuilder {
    access_mode: AccessMode,
    mapping_mode: MappingPermission,
    address_hint: usize,
    size: usize,
    offset: usize,
    enforce_address_hint: bool,
    file_path: Option<FilePath>,
    file_descriptor: Option<FileDescriptor>,
}

impl MemoryMappingBuilder {
    pub fn from_anonymous() -> Self {
        Self {
            access_mode: AccessMode::None,
            mapping_mode: MappingPermission::None,
            address_hint: 0,
            enforce_address_hint: false,
            offset: 0,
            size: 0,
            file_path: None,
            file_descriptor: None,
        }
    }

    pub fn from_fd(file_descriptor: FileDescriptor) -> Self {
        let mut new_self = Self::from_anonymous();
        new_self.file_descriptor = Some(file_descriptor);
        new_self
    }

    pub fn from_file(file_path: &FilePath, value: AccessMode) -> Self {
        let mut new_self = Self::from_anonymous();
        new_self.access_mode = value;
        new_self.file_path = Some(file_path.clone());
        new_self
    }

    pub fn initial_mapping_permission(mut self, value: MappingPermission) -> Self {
        self.mapping_mode = value;
        self
    }

    pub fn mapping_address_hint(mut self, value: usize) -> Self {
        self.address_hint = value;
        self
    }

    pub fn enforce_mapping_address_hint(mut self, value: bool) -> Self {
        self.enforce_address_hint = value;
        self
    }

    pub fn size(mut self, value: usize) -> Self {
        self.size = value;
        self
    }

    pub fn offset(mut self, value: usize) -> Self {
        self.offset = value;
        self
    }

    pub fn create(self) {
        todo!()
    }
}

pub struct MemoryMapping {
    file_descriptor: FileDescriptor,
    file_path: Option<FilePath>,
    base_address: *mut u8,
    size: usize,
}

impl Drop for MemoryMapping {
    fn drop(&mut self) {}
}

impl MemoryMapping {
    pub fn protect(&self, addr: usize) -> ProtectBuilder {
        todo!()
    }

    pub fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }

    pub fn base_address(&self) -> *const u8 {
        self.base_address
    }

    pub fn base_address_mut(&mut self) -> *mut u8 {
        self.base_address
    }

    pub fn file_path(&self) -> &Option<FilePath> {
        &self.file_path
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct ProtectBuilder {}

impl ProtectBuilder {
    pub fn size(mut self, value: usize) -> Self {
        todo!()
    }

    pub fn mapping_permission(mut self, value: MappingPermission) -> Self {
        todo!()
    }

    pub fn apply(self) {
        todo!()
    }
}
