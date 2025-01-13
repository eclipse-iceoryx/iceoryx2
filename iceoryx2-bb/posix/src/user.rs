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

use crate::handle_errno;
use crate::{config::PASSWD_BUFFER_SIZE, system_configuration::*};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_log::fail;

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_system_types::user_name::UserName;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::Struct;
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
    SystemUserNameLengthLongerThanSupportedLength,
    UnknownError(i32)
}

/// Trait to create an [`User`] from an integer by interpreting it as the uid or from a [`String`]
/// or [`str`] by interpreting the value as user name.
pub trait UserExt {
    fn as_user(&self) -> Result<User, UserError>;
}

impl UserExt for u32 {
    fn as_user(&self) -> Result<User, UserError> {
        User::from_uid(*self)
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

#[derive(Debug)]
/// Represents a user in a POSIX system
pub struct User {
    uid: u32,
    gid: u32,
    name: UserName,
    info: String,
    home_dir: String,
    shell: String,
    password: String,
}

#[derive(Debug)]
enum Source {
    Uid(u32),
    UserName(UserName),
}

impl User {
    /// Create an user object from the owner of the process
    pub fn from_self() -> Result<User, UserError> {
        Self::from_uid(unsafe { posix::getuid() })
    }

    /// Create an user object from a given uid. If the uid does not exist an error will be
    /// returned.
    pub fn from_uid(uid: u32) -> Result<User, UserError> {
        Self::from(Source::Uid(uid))
    }

    /// Create an user object from a given user-name. If the user-name does not exist an error will
    /// be returned
    pub fn from_name(user_name: &UserName) -> Result<User, UserError> {
        Self::from(Source::UserName(*user_name))
    }

    /// Return the user id
    pub fn uid(&self) -> u32 {
        self.uid
    }

    /// Return the group id of the users group
    pub fn gid(&self) -> u32 {
        self.gid
    }

    /// Return the name of the user.
    pub fn name(&self) -> &UserName {
        &self.name
    }

    /// Return additional user infos which are defined in the gecos field.
    pub fn info(&self) -> &str {
        self.info.as_str()
    }

    /// Return the users home directory.
    pub fn home_dir(&self) -> &str {
        self.home_dir.as_str()
    }

    /// Return the users default shell.
    pub fn shell(&self) -> &str {
        self.shell.as_str()
    }

    /// Old entry, should contain only 'x'. Returns the password of the user but on modern systems
    /// it should be stored in /etc/shadow
    pub fn password(&self) -> &str {
        self.password.as_str()
    }

    fn extract_entry(
        error_origin: &str,
        field: *mut posix::c_char,
        name: &str,
    ) -> Result<String, UserError> {
        Ok(
            fail!(from error_origin, when unsafe { CStr::from_ptr(field) }.to_str(),
                with UserError::InvalidUTF8SymbolsInEntry,
                "The {} contains invalid UTF-8 symbols.", name)
            .to_string(),
        )
    }

    fn from(source: Source) -> Result<Self, UserError> {
        let mut passwd = posix::passwd::new();
        let mut passwd_ptr: *mut posix::passwd = &mut passwd;
        let mut buffer: [posix::c_char; PASSWD_BUFFER_SIZE] = [0; PASSWD_BUFFER_SIZE];

        let msg = "Unable to acquire user entry";
        let origin = format!("User::from({:?})", source);
        let errno_value = match source {
            Source::UserName(name) => unsafe {
                posix::getpwnam_r(
                    name.as_c_str(),
                    &mut passwd,
                    buffer.as_mut_ptr(),
                    PASSWD_BUFFER_SIZE,
                    &mut passwd_ptr,
                )
            },
            Source::Uid(uid) => unsafe {
                posix::getpwuid_r(
                    uid,
                    &mut passwd,
                    buffer.as_mut_ptr(),
                    PASSWD_BUFFER_SIZE,
                    &mut passwd_ptr,
                )
            },
        }
        .into();

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

        if passwd_ptr.is_null() {
            fail!(from origin, with UserError::UserNotFound, "{} since the user does not exist.", msg);
        }

        let uid = passwd.pw_uid;
        let gid = passwd.pw_gid;
        let name = fail!(from origin, when unsafe { UserName::from_c_str(passwd.pw_name) },
                            with UserError::SystemUserNameLengthLongerThanSupportedLength,
                            "{} since the user name on the system is longer than the supported length of {}.",
                            msg, UserName::max_len());
        let info = Self::extract_entry(&origin, passwd.pw_gecos, "gecos entry")?;
        let home_dir = Self::extract_entry(&origin, passwd.pw_dir, "home directory")?;
        let shell = Self::extract_entry(&origin, passwd.pw_shell, "shell")?;
        let password = Self::extract_entry(&origin, passwd.pw_passwd, "password")?;

        Ok(User {
            uid,
            gid,
            name,
            info,
            home_dir,
            shell,
            password,
        })
    }
}
