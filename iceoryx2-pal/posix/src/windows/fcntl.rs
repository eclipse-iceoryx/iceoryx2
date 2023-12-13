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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use windows_sys::Win32::{
    Foundation::{
        ERROR_ACCESS_DENIED, ERROR_FILE_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_FILE_TOO_LARGE,
        ERROR_PATH_NOT_FOUND, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    },
    Security::{
        GetFileSecurityA, SetFileSecurityA, DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION,
        OWNER_SECURITY_INFORMATION, SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::{
        CreateFileA, GetFileInformationByHandle, GetFileSize, GetFinalPathNameByHandleA,
        LockFileEx, UnlockFileEx, BY_HANDLE_FILE_INFORMATION, CREATE_NEW, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_NORMAL, FILE_NAME_NORMALIZED, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, INVALID_FILE_SIZE, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
        OPEN_ALWAYS, OPEN_EXISTING,
    },
    System::IO::OVERLAPPED,
};

use crate::{
    posix::types::*,
    posix::Errno,
    posix::Struct,
    posix::{
        shm_get_size, win32_security_attributes::from_security_attributes_to_mode, F_GETFD,
        F_GETFL, F_GETLK, F_RDLCK, F_SETFL, F_SETLK, F_SETLKW, F_UNLCK, F_WRLCK, O_NONBLOCK,
        S_IFDIR, S_IFREG,
    },
};

use super::{
    settings::MAX_PATH_LENGTH,
    win32_handle_translator::{FdHandleEntry, HandleTranslator, Win32Handle},
    win32_security_attributes::from_mode_to_security_attributes,
};
use crate::posix;
use crate::win32call;

pub unsafe fn open_with_mode(pathname: *const char, flags: int, mode: mode_t) -> int {
    let access_mode = if flags & posix::O_RDONLY != 0 {
        GENERIC_READ
    } else if flags & posix::O_WRONLY != 0 {
        GENERIC_WRITE
    } else if flags & posix::O_RDWR != 0 {
        GENERIC_WRITE | GENERIC_READ
    } else {
        0
    };

    let shared_mode = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    let open_mode = if flags & posix::O_CREAT != 0 && flags & posix::O_EXCL != 0 {
        CREATE_NEW
    } else if flags & posix::O_CREAT != 0 {
        OPEN_ALWAYS
    } else {
        OPEN_EXISTING
    };

    let security_attributes = from_mode_to_security_attributes(INVALID_HANDLE_VALUE, mode);

    let handle = win32call! {CreateFileA(
        pathname as *const u8,
        access_mode,
        shared_mode,
        &security_attributes,
        open_mode,
        FILE_ATTRIBUTE_NORMAL,
        0,
    ), ignore
        ERROR_FILE_EXISTS, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND,
        ERROR_ACCESS_DENIED, ERROR_FILE_TOO_LARGE};

    if handle == INVALID_HANDLE_VALUE {
        return -1;
    }

    HandleTranslator::get_instance().add(FdHandleEntry::Handle(Win32Handle {
        handle,
        state_handle: INVALID_HANDLE_VALUE,
        lock_state: F_UNLCK,
    }))
}

pub unsafe fn open(pathname: *const char, flags: int) -> int {
    open_with_mode(pathname, flags, 0)
}

impl Struct for BY_HANDLE_FILE_INFORMATION {}

pub unsafe fn fstat(fd: int, buf: *mut stat_t) -> int {
    let mut file_stat = stat_t::new();

    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            let permission_handle;
            if handle.state_handle != INVALID_HANDLE_VALUE {
                permission_handle = handle.state_handle;
                file_stat.st_size = shm_get_size(handle.state_handle) as _;
            } else {
                permission_handle = handle.handle;
                let size = win32call! {GetFileSize(handle.handle, core::ptr::null_mut::<u32>())};
                if size == INVALID_FILE_SIZE {
                    Errno::set(Errno::EINVAL);
                    return -1;
                }

                file_stat.st_size = size as i32;
                file_stat.st_mode = S_IFREG;

                let mut info = BY_HANDLE_FILE_INFORMATION::new();
                if win32call! {GetFileInformationByHandle(handle.handle, &mut info)} == 0 {
                    Errno::set(Errno::EINVAL);
                    return -1;
                }

                file_stat.st_mode = if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                    S_IFDIR
                } else {
                    S_IFREG
                };
            }

            // acquire permissions
            let file_path = match handle_to_file_path(permission_handle) {
                Some(v) => v,
                None => {
                    Errno::set(Errno::EINVAL);
                    return -1;
                }
            };

            match acquire_mode_from_path(&file_path) {
                None => {
                    Errno::set(Errno::EOVERFLOW);
                    -1
                }
                Some(mode) => {
                    file_stat.st_mode |= mode;
                    buf.write(file_stat);

                    0
                }
            }
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub(crate) unsafe fn acquire_mode_from_path(file_path: &[u8]) -> Option<mode_t> {
    const SECURITY_BUFFER_CAPACITY: usize = 1024;
    let mut security_attributes_buffer = [0u8; SECURITY_BUFFER_CAPACITY];
    let mut required_buffer_capacity = 0;
    if win32call!(GetFileSecurityA(
        file_path.as_ptr(),
        DACL_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
        security_attributes_buffer.as_mut_ptr() as *mut void,
        SECURITY_BUFFER_CAPACITY as u32,
        &mut required_buffer_capacity
    )) == 0
    {
        return None;
    }

    let security_attributes = SECURITY_ATTRIBUTES {
        nLength: required_buffer_capacity,
        lpSecurityDescriptor: security_attributes_buffer.as_mut_ptr() as *mut void,
        bInheritHandle: FALSE,
    };

    Some(from_security_attributes_to_mode(&security_attributes))
}

pub unsafe fn fcntl_int(fd: int, cmd: int, arg: int) -> int {
    if cmd == F_GETFL {
        return 0;
    }

    let socket_fd = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Socket(socket)) => socket.fd,
        Some(FdHandleEntry::UdsDatagramSocket(mut socket)) => {
            if cmd == F_SETFL && (arg & O_NONBLOCK != 0) {
                socket.recv_timeout = None;
                HandleTranslator::get_instance().update(FdHandleEntry::UdsDatagramSocket(socket));
            }
            socket.fd
        }
        _ => {
            Errno::set(Errno::ENOTSUP);
            return -1;
        }
    };

    if cmd == F_SETFL {
        let mut mode = if arg & O_NONBLOCK != 0 { 1u32 } else { 0u32 };
        win32call!(winsock windows_sys::Win32::Networking::WinSock::ioctlsocket(socket_fd, windows_sys::Win32::Networking::WinSock::FIONBIO, &mut mode));
        0
    } else {
        Errno::set(Errno::ENOTSUP);
        -1
    }
}

pub unsafe fn fcntl(fd: int, cmd: int, arg: *mut flock) -> int {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            if cmd != F_SETLK && cmd != F_SETLKW && cmd != F_GETLK {
                Errno::set(Errno::EINVAL);
                return -1;
            }

            if cmd == F_GETLK {
                (*arg).l_type = handle.lock_state as i16;
                return 0;
            }

            let lock_type = (*arg).l_type as i32;
            if lock_type != F_UNLCK && lock_type != F_RDLCK && lock_type != F_WRLCK {
                Errno::set(Errno::EINVAL);
                return -1;
            }

            let handle_mut = HandleTranslator::get_instance().get_mut(fd);

            if lock_type == F_UNLCK {
                handle_mut.lock_state = F_UNLCK;
                if win32call! {UnlockFileEx(
                    handle.handle,
                    0,
                    0xffffffff,
                    0xffffffff,
                    core::ptr::null_mut::<OVERLAPPED>(),
                )} == 0
                {
                    Errno::set(Errno::EINVAL);
                    return -1;
                }
                return 0;
            }

            let mut flags = 0;
            if lock_type == F_WRLCK {
                flags |= LOCKFILE_EXCLUSIVE_LOCK;
            }

            if cmd & F_SETLK != 0 {
                flags |= LOCKFILE_FAIL_IMMEDIATELY;
            }

            // TODO: seems to cause "Invalid access to memory location panic"
            if win32call! {LockFileEx(handle.handle, flags, 0, 0xffffffff, 0xffffffff, core::ptr::null_mut::<OVERLAPPED>())}
                == FALSE
            {
                Errno::set(Errno::EINVAL);
                return -1;
            }

            handle_mut.lock_state = lock_type;

            0
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn fcntl2(fd: int, cmd: int) -> int {
    if cmd == F_GETFD {
        match HandleTranslator::get_instance().get(fd) {
            Some(v) => 0,
            None => {
                Errno::set(Errno::EBADF);
                -1
            }
        }
    } else {
        0
    }
}

unsafe fn handle_to_file_path(handle: HANDLE) -> Option<[u8; MAX_PATH_LENGTH]> {
    let mut file_path = [0u8; MAX_PATH_LENGTH];
    let file_path_len = win32call!(GetFinalPathNameByHandleA(
        handle,
        file_path.as_mut_ptr(),
        MAX_PATH_LENGTH as u32,
        FILE_NAME_NORMALIZED
    ));

    if file_path_len as usize > MAX_PATH_LENGTH {
        None
    } else {
        Some(file_path)
    }
}

pub unsafe fn fchmod(fd: int, mode: mode_t) -> int {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            let handle = if handle.state_handle != INVALID_HANDLE_VALUE {
                handle.state_handle
            } else {
                handle.handle
            };

            let file_path = match handle_to_file_path(handle) {
                Some(v) => v,
                None => {
                    Errno::set(Errno::EINVAL);
                    return -1;
                }
            };

            let security_attributes = from_mode_to_security_attributes(handle, mode);

            if win32call!(SetFileSecurityA(
                file_path.as_ptr(),
                DACL_SECURITY_INFORMATION,
                security_attributes.lpSecurityDescriptor
            )) == 0
            {
                Errno::set(Errno::EPERM);
                return -1;
            }

            0
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}
