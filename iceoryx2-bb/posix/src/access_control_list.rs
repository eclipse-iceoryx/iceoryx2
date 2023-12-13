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

//! Access Control Lists provide the ability to set more fine grained permissions to files and
//! directories then the POSIX user, group, others model.
//!
//! # Example
//!
//! ## Apply ACLs to some file
//!
//! ```no_run
//! use iceoryx2_bb_posix::file::*;
//! use iceoryx2_bb_posix::access_control_list::*;
//! use iceoryx2_bb_posix::user::*;
//! use iceoryx2_bb_posix::group::*;
//! use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let file_name = FilePath::new(b"/tmp/test_file").unwrap();
//! let some_file = FileBuilder::new(&file_name)
//!                                   .creation_mode(CreationMode::PurgeAndCreate)
//!                                   .permission(Permission::OWNER_ALL)
//!                                   .create()
//!                                   .expect("failed to create file");
//!
//! let mut acl = AccessControlList::new().expect("failed to create acl");
//! acl.set(Acl::OwningUser, AclPermission::ReadWriteExecute);
//! acl.set(Acl::MaxAccessRightsForNonOwners, AclPermission::ReadExecute);
//! acl.add_user("testuser1".as_user().unwrap().uid(), AclPermission::ReadExecute )
//!             .expect("failed to add user");
//! acl.add_group("testgroup1".as_group().unwrap().gid(), AclPermission::Read)
//!             .expect("failed to add group");
//!
//! acl.apply_to_file_descriptor(unsafe { some_file.file_descriptor().native_handle() })
//!             .expect("failed to apply acl");
//!
//! some_file.remove_self().expect("failed to cleanup file");
//! ```
//!
//! ## Readout ACLs from some file
//!
//! ```no_run
//! use iceoryx2_bb_posix::file::*;
//! use iceoryx2_bb_posix::access_control_list::*;
//! use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let file_name = FilePath::new(b"/tmp/some_file").unwrap();
//! let some_file = FileBuilder::new(&file_name)
//!                                   .creation_mode(CreationMode::PurgeAndCreate)
//!                                   .create()
//!                                   .expect("failed to open file");
//!
//! let acl = AccessControlList::from_file_descriptor(unsafe {some_file.file_descriptor().native_handle() })
//!                                     .expect("failed to create acl");
//! println!("The ACLs as string: {}", acl.as_string().unwrap());
//!
//! let acl_entries = acl.get().expect("failed to get acl entries");
//! for entry in acl_entries {
//!     println!("tag {:?}, id {:?}, permission {:?}", entry.tag(), entry.id(),
//!             entry.permission());
//! }
//! ```

use iceoryx2_bb_container::byte_string::*;
use iceoryx2_bb_elementary::{enum_gen, scope_guard::ScopeGuardBuilder};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::*;
use std::fmt::Debug;

use crate::{config::ACL_LIST_CAPACITY, handle_errno};

pub const ACL_STRING_SIZE: usize = 4096;
pub type AclString = FixedSizeByteString<ACL_STRING_SIZE>;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum AccessControlListCreationError {
    InsufficientMemory,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum AccessControlListCreationFromFdError {
    InvalidFileDescriptor,
    InsufficientMemory,
    NotSupportedByFileSystem,
    UnknownError(i32),
}

enum_gen! {
    AccessControlListAcquireError
  entry:
    InsufficientMemory,
    InvalidValue,
    UnknownError(i32)
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum AccessControlListSetError {
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum AccessControlListApplyError {
    InsufficientPermissions,
    ReadOnlyFileSystem,
    ContainsInvalidValues,
    InvalidFileDescriptor,
    ListTooBig,
    NoSpaceLeft,
    NotSupportedByFileSystem,
    UnknownError(i32),
}

enum_gen! {
    /// The AccessControlListError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward AccessControlListError as more generic return value when a method
    /// returns a AccessControlList***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    AccessControlListError
  generalization:
    FailedToCreate <= AccessControlListCreationError; AccessControlListCreationFromFdError,
    FailedToAcquire <= AccessControlListAcquireError,
    FailedToApply <= AccessControlListSetError; AccessControlListApplyError
}

impl From<AccessControlListSetError> for AccessControlListCreationError {
    fn from(v: AccessControlListSetError) -> Self {
        let AccessControlListSetError::UnknownError(k) = v;
        AccessControlListCreationError::UnknownError(k)
    }
}

/// Trait which allows to convert a type into an [`AccessControlList`]. Is used for instance to
/// convert text ACL representations stored in strings to convert into ACLs.
pub trait AccessControlListExt {
    /// converts type into an [`AccessControlList`]
    fn as_acl(&self) -> Result<AccessControlList, AccessControlListAcquireError>;
}

impl AccessControlListExt for AclString {
    fn as_acl(&self) -> Result<AccessControlList, AccessControlListAcquireError> {
        AccessControlList::from_string(self)
    }
}

/// Represents and ACL tag. Used in [`Entry`] to define the type of entry.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum AclTag {
    OwningUser = posix::ACL_USER_OBJ as _,
    OwningGroup = posix::ACL_GROUP_OBJ as _,
    Other = posix::ACL_OTHER as _,
    MaxAccessRightsForNonOwners = posix::ACL_MASK as _,
    User = posix::ACL_USER as _,
    Group = posix::ACL_GROUP as _,
    Undefined = posix::ACL_UNDEFINED_TAG as _,
}

/// Defines a ACL setting which can be set with [`AccessControlList::set()`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum Acl {
    /// Sets the permissions for the owning user
    OwningUser = posix::ACL_USER_OBJ as _,
    /// Sets the permissions for the owning group
    OwningGroup = posix::ACL_GROUP_OBJ as _,
    /// Sets the permissions for others
    Other = posix::ACL_OTHER as _,
    /// Defines the maximum of access rights which non owners can have
    MaxAccessRightsForNonOwners = posix::ACL_MASK as _,
}

/// Represents and ACL permission entry.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[repr(u32)]
pub enum AclPermission {
    None,
    Read = posix::ACL_READ,
    Write = posix::ACL_WRITE,
    Execute = posix::ACL_EXECUTE,
    ReadWrite = posix::ACL_READ | posix::ACL_WRITE,
    ReadExecute = posix::ACL_READ | posix::ACL_EXECUTE,
    WriteExecute = posix::ACL_WRITE | posix::ACL_EXECUTE,
    ReadWriteExecute = posix::ACL_READ | posix::ACL_WRITE | posix::ACL_EXECUTE,
}

trait AclHandle {
    fn get_handle(&self) -> &posix::acl_entry_t;
}

trait InternalEntry: Debug + AclHandle {
    fn tag_type(&self) -> AclTag {
        let mut tag_type: posix::acl_tag_t = unsafe { std::mem::zeroed() };

        if unsafe { posix::acl_get_tag_type(*self.get_handle(), &mut tag_type) } != 0 {
            fatal_panic!(from self, "This should never happen! An invalid entry handle was provided while acquiring ACL entry tag type.");
        }

        match tag_type {
            posix::ACL_USER => AclTag::User,
            posix::ACL_GROUP => AclTag::Group,
            posix::ACL_USER_OBJ => AclTag::OwningUser,
            posix::ACL_GROUP_OBJ => AclTag::OwningGroup,
            posix::ACL_OTHER => AclTag::Other,
            posix::ACL_MASK => AclTag::MaxAccessRightsForNonOwners,
            _ => AclTag::Undefined,
        }
    }
}

#[derive(Debug)]
struct InternalMutEntry {
    handle: posix::acl_entry_t,
}

impl AclHandle for InternalMutEntry {
    fn get_handle(&self) -> &posix::acl_entry_t {
        &self.handle
    }
}
impl InternalEntry for InternalMutEntry {}

impl InternalMutEntry {
    fn create(
        acl: &mut AccessControlList,
    ) -> Result<InternalMutEntry, AccessControlListCreationError> {
        let mut new_entry = InternalMutEntry {
            handle: 0 as posix::acl_entry_t,
        };

        if unsafe { posix::acl_create_entry(&mut acl.handle, &mut new_entry.handle) } == 0 {
            return Ok(new_entry);
        }

        let msg = "Unable to create entry";
        handle_errno!( AccessControlListCreationError, from new_entry,
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}),", msg, v)
        );
    }

    fn set_tag(&mut self, tag: posix::acl_tag_t) -> Result<(), AccessControlListSetError> {
        if unsafe { posix::acl_set_tag_type(self.handle, tag) } == 0 {
            return Ok(());
        }

        handle_errno!( AccessControlListSetError, from self,
            v => (UnknownError(v as i32), "Unable to set tag {:?} since an unknown error occurred ({}),", tag, v)
        );
    }

    fn set_qualifier(&mut self, id: u32) -> Result<(), AccessControlListSetError> {
        let id_ptr: *const u32 = &id;
        if unsafe { posix::acl_set_qualifier(self.handle, id_ptr as *const posix::void) } == 0 {
            return Ok(());
        }

        handle_errno!( AccessControlListSetError, from self,
            v => (UnknownError(v as i32), "Unable to set uid/gid {} since an unknown error occurred ({}),", id, v)
        );
    }

    fn clear_permissions(
        &mut self,
        permission_set: &posix::acl_permset_t,
    ) -> Result<(), AccessControlListSetError> {
        match unsafe { posix::acl_clear_perms(*permission_set) } == 0 {
            true => Ok(()),
            false => {
                let error_code = Errno::get();
                fail!(from self, with AccessControlListSetError::UnknownError(error_code as i32),
                         "Unable to clear permissions due to an unknown error ({}).", error_code);
            }
        }
    }

    fn set_permission(
        &mut self,
        permission: AclPermission,
    ) -> Result<(), AccessControlListSetError> {
        let mut permission_set: posix::acl_permset_t = unsafe { std::mem::zeroed() };
        match unsafe { posix::acl_get_permset(self.handle, &mut permission_set) } {
            0 => {
                self.clear_permissions(&permission_set)?;

                match permission {
                    AclPermission::None => (),
                    AclPermission::Read => {
                        self.add_permission(&permission_set, posix::ACL_READ)?;
                    }
                    AclPermission::Write => {
                        self.add_permission(&permission_set, posix::ACL_WRITE)?;
                    }
                    AclPermission::Execute => {
                        self.add_permission(&permission_set, posix::ACL_EXECUTE)?;
                    }
                    AclPermission::ReadWrite => {
                        self.add_permission(&permission_set, posix::ACL_READ)?;
                        self.add_permission(&permission_set, posix::ACL_WRITE)?;
                    }
                    AclPermission::ReadExecute => {
                        self.add_permission(&permission_set, posix::ACL_READ)?;
                        self.add_permission(&permission_set, posix::ACL_EXECUTE)?;
                    }
                    AclPermission::WriteExecute => {
                        self.add_permission(&permission_set, posix::ACL_WRITE)?;
                        self.add_permission(&permission_set, posix::ACL_EXECUTE)?;
                    }
                    AclPermission::ReadWriteExecute => {
                        self.add_permission(&permission_set, posix::ACL_READ)?;
                        self.add_permission(&permission_set, posix::ACL_WRITE)?;
                        self.add_permission(&permission_set, posix::ACL_EXECUTE)?;
                    }
                }
            }
            _ => {
                fatal_panic!(from self, "This should never happen! Unable to acquire permission set.");
            }
        }

        Ok(())
    }

    fn add_permission(
        &mut self,
        permission_set: &posix::acl_permset_t,
        permission: posix::acl_perm_t,
    ) -> Result<(), AccessControlListSetError> {
        let value = match permission {
            posix::ACL_READ => "ACL_READ",
            posix::ACL_WRITE => "ACL_WRITE",
            posix::ACL_EXECUTE => "ACL_EXECUTE",
            _ => return Ok(()),
        };

        if unsafe { posix::acl_add_perm(*permission_set, permission) } == 0 {
            return Ok(());
        }

        let error_code = Errno::get();
        fail!(from self, with AccessControlListSetError::UnknownError(error_code as i32),
              "Unable to set permission to {} due to an unknown error ({}).", value, error_code);
    }
}

impl Drop for InternalMutEntry {
    fn drop(&mut self) {
        if self.handle != 0 as posix::acl_entry_t {
            unsafe {
                posix::acl_free(self.handle as *mut posix::void);
            }
        }
    }
}

#[repr(i32)]
enum EntryType {
    First = posix::ACL_FIRST_ENTRY,
    Next = posix::ACL_NEXT_ENTRY,
}

#[derive(Debug)]
struct InternalConstEntry<'a> {
    handle: posix::acl_entry_t,
    parent: &'a AccessControlList,
}

impl AclHandle for InternalConstEntry<'_> {
    fn get_handle(&self) -> &posix::acl_entry_t {
        &self.handle
    }
}
impl InternalEntry for InternalConstEntry<'_> {}

impl<'a> InternalConstEntry<'a> {
    fn create(acl: &AccessControlList, entry_type: EntryType) -> Option<InternalConstEntry> {
        let mut new_entry = InternalConstEntry {
            handle: 0 as posix::acl_entry_t,
            parent: acl,
        };

        let get_entry_result = unsafe {
            posix::acl_get_entry(acl.handle, entry_type as posix::int, &mut new_entry.handle)
        };

        match get_entry_result {
            0 => None,
            1 => Some(new_entry),
            _ => {
                fatal_panic!(from acl, "This should never happen! An invalid handle was provided while acquiring ACL entries.");
            }
        }
    }

    fn qualifier(&self) -> Result<u32, AccessControlListAcquireError> {
        let qualifier = unsafe { posix::acl_get_qualifier(self.handle) };

        if qualifier.is_null() {
            let msg = "Unable to acquire qualifier from entry";
            handle_errno!( AccessControlListAcquireError, from self,
                Errno::EINVAL => (InvalidValue, "{} since the entry tag type does not support qualifiers.", msg),
                Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        let qualifier_value = unsafe { *(qualifier as *const u32) };

        if unsafe { posix::acl_free(qualifier) } == -1 {
            fatal_panic!(from self.parent, "This should never happen! Unable to release previously acquired memory for entry qualifier.");
        }

        Ok(qualifier_value)
    }

    fn permission(&self) -> AclPermission {
        let mut permission_set: posix::acl_permset_t = 0 as posix::acl_permset_t;
        if unsafe { posix::acl_get_permset(self.handle, &mut permission_set) } != 0 {
            fatal_panic!(from self.parent, "This should never happen! Unable to acquire permission set.");
        }

        let has_read = self.get_perm(&permission_set, posix::ACL_READ);
        let has_write = self.get_perm(&permission_set, posix::ACL_WRITE);
        let has_exec = self.get_perm(&permission_set, posix::ACL_EXECUTE);

        if has_read && has_write && has_exec {
            AclPermission::ReadWriteExecute
        } else if has_read && has_write && !has_exec {
            AclPermission::ReadWrite
        } else if has_read && !has_write && has_exec {
            AclPermission::ReadExecute
        } else if !has_read && has_write && has_exec {
            AclPermission::WriteExecute
        } else if has_read && !has_write && !has_exec {
            AclPermission::Read
        } else if !has_read && has_write && !has_exec {
            AclPermission::Write
        } else if !has_read && !has_write && has_exec {
            AclPermission::Execute
        } else {
            AclPermission::None
        }
    }

    fn get_perm(&self, permission_set: &posix::acl_permset_t, permission: posix::uint) -> bool {
        let get_result = unsafe { posix::acl_get_perm(*permission_set, permission) };

        match get_result {
            0 => false,
            1 => true,
            _ => {
                fatal_panic!(from self.parent, "This should never happen! Unable to verify permission");
            }
        }
    }
}

/// Represents and Entry of an [`AccessControlList`].
#[derive(Debug)]
pub struct Entry {
    tag: AclTag,
    id: Option<u32>,
    permission: AclPermission,
}

impl Entry {
    /// The type of entry
    pub fn tag(&self) -> AclTag {
        self.tag
    }

    /// The id of the entry. If the entry is of type [`AclTag::User`] or [`AclTag::OwningUser`]
    /// it represents the user id. If the type is [`AclTag::Group`] or [`AclTag::OwningGroup`]
    /// it represents the group id.
    /// In any other case it returns [`None`].
    pub fn id(&self) -> Option<u32> {
        self.id
    }

    /// Returns the permission of the entry.
    pub fn permission(&self) -> AclPermission {
        self.permission
    }
}

/// Access Control Lists provide the ability to set more fine grained permissions to files and
/// directories then the POSIX user, group, others model.
#[derive(Debug)]
pub struct AccessControlList {
    handle: posix::acl_t,
    entries: Vec<InternalMutEntry>,
}

impl Drop for AccessControlList {
    fn drop(&mut self) {
        if self.handle != 0 as posix::acl_t {
            unsafe { posix::acl_free(self.handle as *mut posix::void) };
        }
    }
}

impl AccessControlList {
    /// Creates a new empty AccessControlList
    pub fn new() -> Result<AccessControlList, AccessControlListCreationError> {
        let mut new_acl = AccessControlList {
            handle: 0 as posix::acl_t,
            entries: vec![],
        };
        new_acl.handle = unsafe { posix::acl_init(ACL_LIST_CAPACITY as i32) };

        match new_acl.handle as _ {
            0 => handle_errno!(AccessControlListCreationError, from new_acl,
               Errno::ENOMEM => (InsufficientMemory, "Unable to create due to insufficient memory."),
               v => (UnknownError(v as i32), "Unable to create since an unknown error occurred ({}).", v)
            ),
            _ => {
                new_acl.add_entry(
                    AclTag::OwningUser as posix::acl_tag_t,
                    None,
                    AclPermission::ReadWriteExecute,
                )?;
                new_acl.add_entry(
                    AclTag::OwningGroup as posix::acl_tag_t,
                    None,
                    AclPermission::Read,
                )?;
                new_acl.add_entry(AclTag::Other as posix::acl_tag_t, None, AclPermission::None)?;
                new_acl.add_entry(
                    AclTag::MaxAccessRightsForNonOwners as posix::acl_tag_t,
                    None,
                    AclPermission::ReadWriteExecute,
                )?;
                Ok(new_acl)
            }
        }
    }

    /// Acquires an AccessControlList from a given file descriptor.
    ///
    /// ```no_run
    /// use iceoryx2_bb_posix::file::*;
    /// use iceoryx2_bb_posix::access_control_list::*;
    /// use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
    /// use iceoryx2_bb_system_types::file_path::FilePath;
    /// use iceoryx2_bb_container::semantic_string::SemanticString;
    ///
    /// let file_name = FilePath::new(b"/tmp/some_file").unwrap();
    /// let some_file = FileBuilder::new(&file_name)
    ///                                   .creation_mode(CreationMode::PurgeAndCreate)
    ///                                   .create()
    ///                                   .expect("failed to open file");
    ///
    /// let acl = AccessControlList::from_file_descriptor(unsafe{some_file.file_descriptor().native_handle()})
    ///                                     .expect("failed to create acl");
    /// ```
    pub fn from_file_descriptor(
        fd: i32,
    ) -> Result<AccessControlList, AccessControlListCreationFromFdError> {
        let mut new_acl = AccessControlList {
            handle: 0 as posix::acl_t,
            entries: vec![],
        };

        new_acl.handle = unsafe { posix::acl_get_fd(fd) };

        if new_acl.handle != 0 as posix::acl_t {
            return Ok(new_acl);
        }

        let msg = "Unable to extract ACLs from file-descriptor";
        handle_errno!(AccessControlListCreationFromFdError, from new_acl,
            Errno::EBADF => (InvalidFileDescriptor, "{} {} due to an invalid file-descriptor.", msg, fd),
            Errno::ENOMEM => (InsufficientMemory, "{} {} due to insufficient memory.", msg, fd),
            Errno::ENOTSUP => (NotSupportedByFileSystem, "{} {} since it is not supported by the file-system.", msg, fd),
            v => (UnknownError(v as i32), "{} {} since an unknown error occurred ({}).", msg, fd, v)
        );
    }

    /// Creates an acl from a string. The string must follow the ACL syntax and must consist only
    /// of ASCII characters. See [`AccessControlList::as_string()`] for an example.
    pub fn from_string(
        value: &AclString,
    ) -> Result<AccessControlList, AccessControlListAcquireError> {
        let mut new_acl = AccessControlList {
            handle: 0 as posix::acl_t,
            entries: vec![],
        };

        let msg = "Unable to create ACL from text";
        new_acl.handle = unsafe { posix::acl_from_text(value.as_c_str()) };
        if new_acl.handle != 0 as posix::acl_t {
            return Ok(new_acl);
        }

        handle_errno!( AccessControlListAcquireError, from new_acl,
            Errno::EINVAL => (InvalidValue, "{} \"{}\" since the text cannot be translated.", msg, value),
            Errno::ENOMEM => (InsufficientMemory, "{} \"{}\" due to insufficient memory.", msg, value),
            v => (UnknownError(v as i32), "{} \"{}\" since an unknown error occurred ({}).", msg, value, v)
        );
    }

    /// Returns the current AccessControlList as string. Can be used to construct a new
    /// AccessControlList with [`AccessControlList::from_string()`].
    ///
    /// ```ignore
    /// use iceoryx2_bb_posix::access_control_list::*;
    /// use iceoryx2_bb_posix::user::*;
    /// use iceoryx2_bb_posix::group::*;
    ///
    /// let mut acl = AccessControlList::new().expect("failed to create acl");
    /// acl.add_user("testuser2".as_user().unwrap().uid(), AclPermission::ReadExecute )
    ///             .expect("failed to add user");
    /// acl.add_group("testgroup2".as_group().unwrap().gid(), AclPermission::Read)
    ///             .expect("failed to add group");
    ///
    /// let acl_string = acl.as_string().unwrap();
    ///
    /// let acl_from_string = AccessControlList::from_string(&acl_string).unwrap();
    /// ```
    //
    pub fn as_string(&self) -> Result<AclString, AccessControlListAcquireError> {
        let msg = "Unable to convert acl to string";
        let acl_value = ScopeGuardBuilder::new(std::ptr::null::<posix::char>())
            .on_init(|v| {
                *v = unsafe { posix::acl_to_text(self.handle, std::ptr::null_mut::<isize>()) };
                match !(*v).is_null()  {
                    true => Ok(()),
                    false => {
                        handle_errno!(AccessControlListAcquireError, from self,
                            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).",msg, v)
                        );
                    }
                }
            })
            .on_drop(|v| match unsafe { posix::acl_free(*v as *mut posix::void) } {
                0 => (),
                _ => {
                    fatal_panic!(from self, "{} since a fatal failure occured while cleaning up acquired acl text. Memory Leak?", msg);
                }
            })
            .create()?;

        Ok(
            fail!(from self, when unsafe { AclString::from_c_str(*acl_value.get() as *mut posix::char) },
                            with AccessControlListAcquireError::InvalidValue,
                            "{} since the acl text length exceeds the maximum supported AclString capacity of ({}).",
                            msg, ACL_STRING_SIZE),
        )
    }

    /// Returns all entries of an AccessControlList.
    pub fn get(&self) -> Result<Vec<Entry>, AccessControlListAcquireError> {
        let mut entries: Vec<Entry> = vec![];

        let mut entry_type = EntryType::First;
        loop {
            match InternalConstEntry::create(self, entry_type) {
                Some(v) => {
                    let tag_type = v.tag_type();
                    entries.push(Entry {
                        tag: tag_type,
                        permission: v.permission(),
                        id: if tag_type == AclTag::User || tag_type == AclTag::Group {
                            Some(v.qualifier()?)
                        } else {
                            None
                        },
                    });
                }
                None => {
                    return Ok(entries);
                }
            }
            entry_type = EntryType::Next;
        }
    }

    /// Sets an ACL setting defined in [`Acl`].
    ///
    /// ```ignore
    /// use iceoryx2_bb_posix::access_control_list::*;
    ///
    /// let mut acl = AccessControlList::new().expect("failed to create acl");
    /// acl.set(Acl::OwningUser, AclPermission::ReadWrite).unwrap();
    /// ```
    pub fn set(
        &mut self,
        setting: Acl,
        permission: AclPermission,
    ) -> Result<(), AccessControlListCreationError> {
        for entry in &mut self.entries {
            if entry.tag_type() as i32 == setting as i32 {
                entry.set_permission(permission)?;
                return Ok(());
            }
        }

        self.add_entry(setting as _, None, permission)?;
        Ok(())
    }

    /// Adds a new user to the AccessControlList with a specified permission.
    ///
    /// ```ignore
    /// use iceoryx2_bb_posix::access_control_list::*;
    /// use iceoryx2_bb_posix::user::*;
    ///
    /// let mut acl = AccessControlList::new().expect("failed to create acl");
    /// acl.add_user("testuser1".as_user().unwrap().uid(), AclPermission::ReadExecute )
    ///             .expect("failed to add user");
    /// ```
    pub fn add_user(
        &mut self,
        uid: u32,
        permission: AclPermission,
    ) -> Result<(), AccessControlListCreationError> {
        self.add_entry(AclTag::User as posix::acl_tag_t, Some(uid), permission)
    }

    /// Adds a new user to the AccessControlList with a specified permission.
    ///
    /// ```ignore
    /// use iceoryx2_bb_posix::access_control_list::*;
    /// use iceoryx2_bb_posix::group::*;
    ///
    /// let mut acl = AccessControlList::new().expect("failed to create acl");
    /// acl.add_group("testgroup2".as_group().unwrap().gid(), AclPermission::ReadExecute )
    ///             .expect("failed to add group");
    /// ```
    pub fn add_group(
        &mut self,
        gid: u32,
        permission: AclPermission,
    ) -> Result<(), AccessControlListCreationError> {
        self.add_entry(AclTag::Group as posix::acl_tag_t, Some(gid), permission)
    }

    /// Applies the AccessControlList to a file descriptor.
    pub fn apply_to_file_descriptor(&self, fd: i32) -> Result<(), AccessControlListApplyError> {
        if unsafe { posix::acl_valid(self.handle) } == -1 {
            fail!(from self, with AccessControlListApplyError::ContainsInvalidValues, "Unable to apply the AccessControlList to file-descriptor {} since it contains invalid values.", fd);
        }

        if unsafe { posix::acl_set_fd(fd, self.handle) } == 0 {
            return Ok(());
        }

        let msg = "Unable to apply the AccessControlList to file-descriptor";
        handle_errno!(AccessControlListApplyError, from self,
            Errno::EBADF => (InvalidFileDescriptor, "{} {} due to an invalid file-descriptor.", msg, fd),
            Errno::EINVAL => (ListTooBig, "{} {} since it contains more entries than the file can obtain.", msg, fd),
            Errno::ENOSPC => (NoSpaceLeft, "{} {} since there is no space left on the target device.", msg, fd),
            Errno::ENOTSUP => (NotSupportedByFileSystem, "{} {} since it is not supported by the file-system.", msg, fd),
            Errno::EPERM => (InsufficientPermissions, "{} {} due to insufficient permissions.", msg, fd),
            Errno::EROFS => (ReadOnlyFileSystem, "{} {} since the file-system is read-only.", msg, fd),
            v => (UnknownError(v as i32), "{} {} since an unknown error occurred ({}).", msg, fd, v)
        );
    }

    fn add_entry(
        &mut self,
        tag: posix::acl_tag_t,
        id: Option<u32>,
        permission: AclPermission,
    ) -> Result<(), AccessControlListCreationError> {
        let mut new_entry = InternalMutEntry::create(self)?;
        new_entry.set_tag(tag)?;
        if id.is_some() {
            new_entry.set_qualifier(id.unwrap())?;
        }
        new_entry.set_permission(permission)?;
        self.entries.push(new_entry);

        Ok(())
    }
}
