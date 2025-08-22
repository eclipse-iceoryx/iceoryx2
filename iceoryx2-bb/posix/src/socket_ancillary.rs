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

//! The [`SocketAncillary`] can be sent with
//! [`crate::unix_datagram_socket::UnixDatagramSender::try_send_msg()`] and
//! received via
//! [`crate::unix_datagram_socket::UnixDatagramReceiver::try_receive_msg()`]. One can use it to exchange
//! file descriptors between processes or authenticate at another process by sending
//! [`SocketCred`] containing the process pid, uid and gid.
//!
//! # Example
//!
//! ```no_run
//! use iceoryx2_bb_posix::unix_datagram_socket::*;
//! use iceoryx2_bb_posix::socket_ancillary::*;
//! use iceoryx2_bb_posix::file::*;
//! use iceoryx2_bb_posix::file_descriptor::*;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let socket_name = FilePath::new(b"credential_socket_example").unwrap();
//! let file_name = FilePath::new(b"some_funky_file").unwrap();
//! let receiver = UnixDatagramReceiverBuilder::new(&socket_name)
//!                     .creation_mode(CreationMode::PurgeAndCreate)
//!                     .create().unwrap();
//!
//! let sender = UnixDatagramSenderBuilder::new(&socket_name)
//!                     .create().unwrap();
//!
//! let file = FileBuilder::new(&file_name)
//!                     .creation_mode(CreationMode::PurgeAndCreate)
//!                     .create().unwrap();
//!
//! let mut msg = SocketAncillary::new();
//!
//! // send file descriptor of file to another process
//! if msg.add_fd(file.file_descriptor().clone()) {
//!     println!("No more space left in message for another file descriptor.");
//! }
//! // SocketCred::new fills the struct with the process pid, uid and gid, used for
//! // authentication
//! msg.set_creds(&SocketCred::new());
//!
//! sender.try_send_msg(&mut msg).unwrap();
//!
//! // is usually implemented in the process where the authentication is performed
//! let mut recv_msg = SocketAncillary::new();
//!
//! receiver.blocking_receive_msg(&mut recv_msg).unwrap();
//! match recv_msg.get_creds() {
//!     Some(cred) => { println!("received credentials {}", cred); }
//!     None => { println!("Does not happen when `support_credentials(true)` is set."); }
//! };
//!
//! let mut fd_vec = recv_msg.extract_fds();
//! // open the file to the received file descriptor
//! if fd_vec.is_empty() {
//!     println!("No file descriptors received.");
//! }
//! let recv_file = File::from_file_descriptor(fd_vec.remove(0));
//!
//! // cleanup
//! File::remove(&file_name);
//! ```
use crate::{
    file_descriptor::FileDescriptor, group::Gid, process::*,
    unix_datagram_socket::UnixDatagramReceiver, user::Uid,
};
use core::{fmt::Display, marker::PhantomPinned};
use iceoryx2_bb_log::warn;
use iceoryx2_pal_posix::{posix::MemZeroedStruct, *};

/// Defines the maximum amount of [`FileDescriptor`]s which can be sent with a single message.
pub const MAX_FILE_DESCRIPTORS_PER_MESSAGE: usize = posix::SCM_MAX_FD as usize;

const SIZE_OF_CRED: usize = core::mem::size_of::<posix::ucred>();
const SIZE_OF_FD: usize = core::mem::size_of::<i32>();
const IOVEC_BUFFER_CAPACITY: usize = 1;
const BUFFER_CAPACITY: usize = 3072;
pub(crate) const CMSG_SOCKET_LEVEL: posix::int = posix::SOL_SOCKET;

fn buffer_capacity() -> usize {
    unsafe {
        (posix::CMSG_SPACE(SIZE_OF_FD as _) as usize) * MAX_FILE_DESCRIPTORS_PER_MESSAGE
            + posix::CMSG_SPACE(SIZE_OF_CRED as _) as usize
    }
}

/// Credentials which contain the process id, user id and group id. Those credentials can be
/// sent via [`crate::unix_datagram_socket::UnixDatagramSender`] to a
/// [`crate::unix_datagram_socket::UnixDatagramReceiver`] to authenticate
/// an application.
/// The pid, uid and gid are verified by the operating system.
///
/// # Example
///
/// See [`SocketAncillary`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SocketCred {
    pid: ProcessId,
    uid: Uid,
    gid: Gid,
}

impl Default for SocketCred {
    fn default() -> Self {
        Self {
            pid: Process::from_self().id(),
            uid: Uid::new_from_native(unsafe { posix::getuid() }),
            gid: Gid::new_from_native(unsafe { posix::getgid() }),
        }
    }
}

impl SocketCred {
    /// Creates a new [`SocketCred`] with the pid, uid and gid of the process.
    pub fn new() -> SocketCred {
        Self::default()
    }

    /// Overrides the current pid
    pub fn set_pid(&mut self, pid: ProcessId) {
        self.pid = pid;
    }

    /// Overrides the current uid
    pub fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }

    /// Overrides the current gid
    pub fn set_gid(&mut self, gid: Gid) {
        self.gid = gid;
    }

    pub fn get_pid(&self) -> ProcessId {
        self.pid
    }

    pub fn get_uid(&self) -> Uid {
        self.uid
    }

    pub fn get_gid(&self) -> Gid {
        self.gid
    }
}

impl Display for SocketCred {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SocketCred {{ pid: {:?}, uid: {}, gid: {} }}",
            self.pid, self.uid, self.gid
        )
    }
}

/// Represents a message which can be sent or received via
/// [`crate::unix_datagram_socket::UnixDatagramSender::try_send_msg()`] or
/// [`crate::unix_datagram_socket::UnixDatagramReceiver::try_receive_msg()`].
pub struct SocketAncillary {
    message_buffer: [u8; BUFFER_CAPACITY],
    iovec_buffer: [u8; IOVEC_BUFFER_CAPACITY],
    iovec: posix::iovec,
    message: posix::msghdr,
    file_descriptors: Vec<FileDescriptor>,
    credentials: Option<SocketCred>,
    is_prepared_for_send: bool,
    set_memory_to_zero_first: bool,
    _alignment: [posix::msghdr; 0],
    _pin: PhantomPinned,
}

impl Display for SocketAncillary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let cred = if self.credentials.is_some() {
            format!("{}", self.credentials.as_ref().unwrap())
        } else {
            "None".to_string()
        };

        write!(
            f,
            "SocketAncillary {{ credentials: {}, file descriptors: {:?} }}",
            cred, self.file_descriptors
        )
    }
}

struct UdsMsgHeader {
    header: *mut posix::cmsghdr,
}

impl UdsMsgHeader {
    fn level(&mut self, value: i32) {
        unsafe { (*self.header).cmsg_level = value };
    }

    fn header_type(&mut self, value: i32) {
        unsafe { (*self.header).cmsg_type = value };
    }

    fn assign_via_memcpy<T>(&mut self, value: &T) {
        let memcpy_required_space = core::mem::size_of::<T>();
        let required_space = unsafe { posix::CMSG_LEN(memcpy_required_space as _) };

        unsafe {
            posix::memcpy(
                posix::CMSG_DATA(self.header) as *mut posix::void,
                (value as *const T) as *const posix::void,
                memcpy_required_space,
            )
        };
        unsafe { (*self.header).cmsg_len = required_space as _ };
    }

    fn assign_from_slice<T>(&mut self, value: &[T]) {
        let memcpy_required_space = core::mem::size_of_val(value);
        let required_space = unsafe { posix::CMSG_LEN(memcpy_required_space as _) };

        unsafe {
            posix::memcpy(
                posix::CMSG_DATA(self.header) as *mut posix::void,
                value.as_ptr() as *const posix::void,
                memcpy_required_space,
            )
        };
        unsafe { (*self.header).cmsg_len = required_space as _ };
    }
}

impl Default for SocketAncillary {
    fn default() -> Self {
        let mut new_self = Self {
            message_buffer: [0u8; BUFFER_CAPACITY],
            iovec_buffer: [0u8; IOVEC_BUFFER_CAPACITY],
            iovec: unsafe { core::mem::zeroed() },
            message: posix::msghdr {
                msg_name: core::ptr::null_mut::<posix::void>(),
                msg_namelen: 0,
                msg_iov: core::ptr::null_mut(),
                msg_iovlen: IOVEC_BUFFER_CAPACITY as _,
                msg_control: core::ptr::null_mut::<posix::void>(),
                msg_controllen: buffer_capacity() as _,
                msg_flags: 0,
            },
            file_descriptors: vec![],
            credentials: None,
            is_prepared_for_send: false,
            set_memory_to_zero_first: false,
            _alignment: unsafe { core::mem::zeroed() },
            _pin: PhantomPinned,
        };

        new_self
            .iovec
            .set_base(new_self.iovec_buffer.as_mut_ptr() as *mut posix::void);
        new_self.iovec.set_len(IOVEC_BUFFER_CAPACITY);

        new_self.message.msg_iov = new_self.iovec.as_mut_ptr();
        new_self.message.msg_control = new_self.message_buffer.as_mut_ptr() as *mut posix::void;

        new_self
    }
}

impl SocketAncillary {
    /// creates a new empty [`SocketAncillary`] struct
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_full(&self) -> bool {
        self.file_descriptors.len() == posix::SCM_MAX_FD as usize
    }

    pub fn is_empty(&self) -> bool {
        self.file_descriptors.is_empty() && self.credentials.is_none()
    }

    /// Takes ownership of a file descriptor which can be transferred to another process
    pub fn add_fd(&mut self, fd: FileDescriptor) -> bool {
        if self.file_descriptors.len() < posix::SCM_MAX_FD as usize {
            self.file_descriptors.push(fd);
            self.is_prepared_for_send = false;
            return true;
        }
        false
    }

    /// Sets the credentials of the message.
    /// This can be used to override the credentials if the process
    /// has the permission. Otherwise the operating system will reject the message.
    pub fn set_creds(&mut self, creds: &SocketCred) {
        self.credentials = Some(*creds);
        self.is_prepared_for_send = false;
    }

    /// Returns the contained [`FileDescriptor`] vector
    pub fn get_fds(&self) -> &Vec<FileDescriptor> {
        &self.file_descriptors
    }

    /// Returns the contained [`SocketCred`]
    pub fn get_creds(&self) -> Option<SocketCred> {
        self.credentials
    }

    /// Destroys [`SocketAncillary`] and extract the contained [`FileDescriptor`]
    pub fn extract_fds(self) -> Vec<FileDescriptor> {
        self.file_descriptors
    }

    /// Clears all contained [`FileDescriptor`] and [`SocketCred`]
    pub fn clear(&mut self) {
        self.file_descriptors.clear();
        self.credentials = None;
        self.message.msg_controllen = buffer_capacity() as _;
        self.is_prepared_for_send = false;
        self.set_memory_to_zero_first = true;
    }

    pub(crate) fn len(&self) -> usize {
        self.message.msg_controllen as _
    }

    pub(crate) fn extract_received_data(&mut self, receiver: &UnixDatagramReceiver) {
        let mut cmsghdr = unsafe { posix::CMSG_FIRSTHDR(&self.message) };

        loop {
            if cmsghdr.is_null() {
                break;
            }

            if unsafe { (*cmsghdr).cmsg_level != CMSG_SOCKET_LEVEL } {
                warn!(from receiver, "A cmsghdr with the wrong cmsg_level was received - expected {}, received {}.",
                    unsafe{(*cmsghdr).cmsg_level}, CMSG_SOCKET_LEVEL);
                continue;
            }

            match unsafe { (*cmsghdr).cmsg_type } {
                posix::SCM_RIGHTS => {
                    let mut i = 0;
                    if self.len() % SIZE_OF_FD != 0 {
                        warn!(from receiver, "Received an incomplete set of file descriptors.")
                    }

                    while i < self.len() {
                        let mut raw_fd: i32 = 0;
                        unsafe {
                            posix::memcpy(
                                (&mut raw_fd as *mut i32) as *mut posix::void,
                                posix::CMSG_DATA(cmsghdr).add(i) as *const posix::void,
                                SIZE_OF_FD,
                            )
                        };
                        if raw_fd == 0 {
                            break;
                        }

                        if let Some(fd) = FileDescriptor::new(raw_fd) {
                            self.file_descriptors.push(fd);
                        } else {
                            warn!(from receiver, "An invalid file descriptor was received and will be ignored.");
                        }

                        i += SIZE_OF_FD;
                    }
                }
                posix::SCM_CREDENTIALS => {
                    let mut raw_cred = posix::ucred::new_zeroed();
                    unsafe {
                        posix::memcpy(
                            (&mut raw_cred as *mut posix::ucred) as *mut posix::void,
                            posix::CMSG_DATA(cmsghdr) as *const posix::void,
                            SIZE_OF_CRED,
                        )
                    };

                    self.credentials = Some(SocketCred {
                        pid: ProcessId::new(raw_cred.pid),
                        uid: Uid::new_from_native(raw_cred.uid),
                        gid: Gid::new_from_native(raw_cred.gid),
                    });
                }
                v => {
                    warn!(from receiver, "A cmsghdr with an unknown cmsg_type ({}) was received.", v);
                }
            }

            cmsghdr = unsafe { posix::CMSG_NXTHDR(&self.message, cmsghdr) };
        }
    }

    pub(crate) fn prepare_for_send(&mut self) {
        if self.is_prepared_for_send {
            return;
        }

        if self.set_memory_to_zero_first {
            self.message_buffer = [0u8; BUFFER_CAPACITY];
            self.iovec_buffer = [0u8; IOVEC_BUFFER_CAPACITY];
        }

        let mut controllen: usize = 0;
        if !self.file_descriptors.is_empty() {
            let mut raw_fd = vec![];
            for fd in &self.file_descriptors {
                raw_fd.push(unsafe { fd.native_handle() });
            }

            let mut header = self.header_from(unsafe { posix::CMSG_FIRSTHDR(&self.message) }).expect(
                "Bug in implementation. There should be always enough space to acquire cmsghdr for file descriptors.",
            );

            header.header_type(posix::SCM_RIGHTS);
            header.assign_from_slice(raw_fd.as_slice());
            controllen +=
                unsafe { posix::CMSG_SPACE((SIZE_OF_FD * self.file_descriptors.len()) as _) }
                    as usize;
        }

        if self.credentials.is_some() {
            let credentials = self.credentials.unwrap();
            let mut header = if !self.file_descriptors.is_empty() {
                self.header_from(unsafe { posix::CMSG_NXTHDR(&self.message, posix::CMSG_FIRSTHDR(&self.message)) }).expect(
                    "Bug in implementation. There should be always enough space to acquire cmsghdr for credentials.",
                )
            } else {
                self.header_from(unsafe { posix::CMSG_FIRSTHDR(&self.message) }).expect(
                    "Bug in implementation. There should be always enough space to acquire cmsghdr for credentials.",
                )
            };

            header.header_type(posix::SCM_CREDENTIALS);
            header.assign_via_memcpy(&credentials);
            controllen += unsafe { posix::CMSG_SPACE(SIZE_OF_CRED as _) } as usize;
        }

        self.message.msg_controllen = controllen as _;
    }

    pub(crate) fn get(&self) -> *const posix::msghdr {
        &self.message
    }

    pub(crate) fn get_mut(&mut self) -> *mut posix::msghdr {
        &mut self.message
    }

    fn header_from(&mut self, header: *mut posix::cmsghdr) -> Option<UdsMsgHeader> {
        match !header.is_null() {
            true => {
                let mut header = UdsMsgHeader { header };
                header.level(CMSG_SOCKET_LEVEL);
                unsafe { (*header.header).cmsg_type = 0 };
                unsafe { (*header.header).cmsg_len = 0 };

                Some(header)
            }
            false => None,
        }
    }
}
