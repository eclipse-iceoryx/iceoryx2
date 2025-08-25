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

//! Provides the trait [`UserExt`] to create users from strings by interpreting them as user
//! name or from unsigned integers by interpreting them as user id. The [`User`] struct provides
//! access to the properties of a POSIX user.
//!
//! # Example
//!
//! ```ignore
//! use iceoryx2_bb_posix::user::*;
//!
//! let myself = User::from_self().expect("failed to get user");
//! let root = "root".as_user().expect("failed to get root user");
//!
//! println!("I am {:?} and the root account is {:?}", myself, root);
//! println!("my shell: {}", myself.shell());
//! println!("my homedir: {}", myself.home_dir());
//! println!("my gecos: {}", myself.info());
//! ```

use core::ffi::CStr;
use core::fmt::Display;

use crate::group::Gid;
use crate::handle_errno;
use crate::{config::PASSWD_BUFFER_SIZE, system_configuration::*};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_log::{fail, warn};

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_bb_system_types::user_name::UserName;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::*;

enum_gen! { UserError
  entry:
    UserNotFound,
    Interrupt,
    IOerror,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    InsufficientBufferSize,
    InvalidUTF8SymbolsInEntry,
    InvalidSymbolsInPathEntry,
    InvalidSymbolsInShellPath,
    ConfigPathIsTooLong,
    SystemUserNameLengthLongerThanSupportedLength,
    UserIdOutOfRange,
    UnknownError(i32)
}

/// Trait to create an [`User`] from an integer by interpreting it as the uid or from a [`String`]
/// or [`str`] by interpreting the value as user name.
pub trait UserExt {
    fn as_user(&self) -> Result<User, UserError>;
}

impl UserExt for u32 {
    fn as_user(&self) -> Result<User, UserError> {
        let uid = Uid::new(*self).ok_or(UserError::UserIdOutOfRange)?;
        User::from_uid(uid)
    }
}

impl UserExt for String {
    fn as_user(&self) -> Result<User, UserError> {
        User::from_name(
            &fail!(from "String::as_user()", when UserName::new(self.as_bytes()),
                with UserError::InvalidUTF8SymbolsInEntry,
                "Failed to create user object since the name \"{}\" contains invalid characters.",
                self),
        )
    }
}

impl UserExt for &str {
    fn as_user(&self) -> Result<User, UserError> {
        User::from_name(
            &fail!(from "&str::as_user()", when UserName::new(self.as_bytes()),
                with UserError::InvalidUTF8SymbolsInEntry,
                "Failed to create user object since the name \"{}\" contains invalid characters.",
                self),
        )
    }
}

impl UserExt for UserName {
    fn as_user(&self) -> Result<User, UserError> {
        User::from_name(self)
    }
}

/// Contains additional details of the [`User`] that might be not available on every platform or
/// on every platform configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserDetails {
    gid: Gid,
    name: UserName,
    home_dir: Path,
    config_dir: Path,
    shell: FilePath,
}

impl UserDetails {
    /// Return the group id of the users group
    pub fn gid(&self) -> Gid {
        self.gid
    }

    /// Return the name of the user.
    pub fn name(&self) -> &UserName {
        &self.name
    }

    /// Return the users home directory.
    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }

    /// Returns the users config directory.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Return the users default shell.
    pub fn shell(&self) -> &FilePath {
        &self.shell
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Uid {
    uid: u32,
}

trait UidInRange {
    fn uid_in_range(other: u32) -> bool;
}

impl UidInRange for u32 {
    fn uid_in_range(_other: u32) -> bool {
        true
    }
}

impl UidInRange for i32 {
    fn uid_in_range(other: u32) -> bool {
        other <= i32::MAX as u32
    }
}

impl UidInRange for u16 {
    fn uid_in_range(other: u32) -> bool {
        other <= u16::MAX as u32
    }
}

impl Uid {
    pub fn new(uid: u32) -> Option<Self> {
        if posix::uid_t::uid_in_range(uid) {
            Some(Self { uid })
        } else {
            None
        }
    }

    pub fn new_from_native(uid: posix::uid_t) -> Self {
        Self { uid: uid as _ }
    }

    pub fn value(&self) -> u32 {
        self.uid
    }

    pub fn to_native(&self) -> posix::uid_t {
        // NOTE: this is safe since the range is checked on construction
        self.uid as _
    }
}

impl Display for Uid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.uid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a user in a POSIX system
pub struct User {
    uid: Uid,
    details: Option<UserDetails>,
}

impl User {
    /// Create an user object from the owner of the process
    pub fn from_self() -> Result<User, UserError> {
        Self::from_uid(Uid::new_from_native(unsafe { posix::getuid() }))
    }

    /// Create an user object from a given uid. If the uid does not exist an error will be
    /// returned.
    pub fn from_uid(uid: Uid) -> Result<User, UserError> {
        let msg = "Unable to acquire user entry";
        let origin = format!("User::from_uid({uid})");

        let mut passwd = posix::passwd::new_zeroed();
        let mut passwd_ptr: *mut posix::passwd = &mut passwd;
        let mut buffer: [posix::c_char; PASSWD_BUFFER_SIZE] = [0; PASSWD_BUFFER_SIZE];

        let errno_value = unsafe {
            posix::getpwuid_r(
                uid.to_native(),
                &mut passwd,
                buffer.as_mut_ptr(),
                PASSWD_BUFFER_SIZE,
                &mut passwd_ptr,
            )
        }
        .into();

        match Self::handle_errno(errno_value, msg, &origin) {
            Ok(()) => Self::extract_user_details(msg, &origin, passwd_ptr, &mut passwd),
            Err(UserError::UnknownError(e)) => {
                warn!(from origin,
                    "{} details since an unknown failure occurred while reading the underlying POSIX user database `/etc/passwd` ({}). It is possible that those information are not available on this platform or platform-configuration.",
                    msg, e);
                Ok(User { uid, details: None })
            }
            Err(e) => {
                fail!(from origin, with e,
                    "{} details since the underlying POSIX user database `/etc/passwd` could not be read ({:?}).",
                    msg, e);
            }
        }
    }

    /// Create an user object from a given user-name. If the user-name does not exist an error will
    /// be returned
    pub fn from_name(user_name: &UserName) -> Result<User, UserError> {
        let msg = "Unable to acquire user entry";
        let origin = format!("User::from_name({user_name})");

        let mut passwd = posix::passwd::new_zeroed();
        let mut passwd_ptr: *mut posix::passwd = &mut passwd;
        let mut buffer: [posix::c_char; PASSWD_BUFFER_SIZE] = [0; PASSWD_BUFFER_SIZE];

        let errno_value = unsafe {
            posix::getpwnam_r(
                user_name.as_c_str(),
                &mut passwd,
                buffer.as_mut_ptr(),
                PASSWD_BUFFER_SIZE,
                &mut passwd_ptr,
            )
        }
        .into();

        Self::handle_errno(errno_value, msg, &origin)?;
        Self::extract_user_details(msg, &origin, passwd_ptr, &mut passwd)
    }

    /// Return the user id
    pub fn uid(&self) -> Uid {
        self.uid
    }

    /// Returns the optional [`UserDetails`] that might be not available on every platform or
    /// on every platform configuration.
    pub fn details(&self) -> Option<&UserDetails> {
        self.details.as_ref()
    }

    fn handle_errno(errno_value: Errno, msg: &str, origin: &str) -> Result<(), UserError> {
        handle_errno!(UserError, from origin,
            errno_source errno_value,
            continue_on_success,
            success Errno::ESUCCES => (),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received", msg ),
            Errno::EIO => (IOerror, "{} due to an I/O error.", msg),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process file handle limit is reached.", msg ),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide file handle limit is reached.", msg),
            Errno::ERANGE => (InsufficientBufferSize, "{} since insufficient storage was provided. Max buffer size should be: {}", msg, Limit::MaxSizeOfPasswordBuffer.value()),
            v => (UnknownError(v as i32), "{} due to an unknown error ({}).", msg, v)
        );

        Ok(())
    }

    fn extract_entry(
        error_origin: &str,
        field: *mut posix::c_char,
        error_msg: &str,
        name: &str,
    ) -> Result<String, UserError> {
        Ok(
            fail!(from error_origin, when unsafe { CStr::from_ptr(field) }.to_str(),
                with UserError::InvalidUTF8SymbolsInEntry,
                "{} since the {} contains invalid UTF-8 symbols.", error_msg, name)
            .to_string(),
        )
    }

    fn extract_user_details(
        msg: &str,
        origin: &str,
        passwd_ptr: *mut posix::passwd,
        passwd: &mut posix::passwd,
    ) -> Result<Self, UserError> {
        if passwd_ptr.is_null() {
            fail!(from origin, with UserError::UserNotFound, "{} since the user does not exist.", msg);
        }

        let uid = Uid::new_from_native(passwd.pw_uid);
        let gid = Gid::new_from_native(passwd.pw_gid);
        let name = fail!(from origin, when unsafe { UserName::from_c_str(passwd.pw_name) },
                            with UserError::SystemUserNameLengthLongerThanSupportedLength,
                            "{} since the user name on the system is longer than the supported length of {}.",
                            msg, UserName::max_len());
        let home_dir_raw = Self::extract_entry(origin, passwd.pw_dir, msg, "home directory")?;
        let home_dir = fail!(from origin,
                        when Path::new(home_dir_raw.as_bytes()),
                        with UserError::InvalidSymbolsInPathEntry,
                        "{} since the user home dir path \"{}\" contains invalid path symbols.", msg, home_dir_raw);
        let mut config_dir = home_dir.clone();
        fail!(from origin,
              when config_dir.add_path_entry(&get_user_config_path()),
              with UserError::ConfigPathIsTooLong,
              "{} since the user config directory path is too long.", msg);

        let shell_raw = Self::extract_entry(origin, passwd.pw_shell, msg, "shell")?;
        let shell = fail!(from origin,
            when FilePath::new(shell_raw.as_bytes()),
            with UserError::InvalidSymbolsInShellPath,
            "{} since the user shell path \"{}\" contains invalid path symbols", msg, shell_raw);

        Ok(User {
            uid,
            details: Some(UserDetails {
                gid,
                name,
                home_dir,
                config_dir,
                shell,
            }),
        })
    }
}
