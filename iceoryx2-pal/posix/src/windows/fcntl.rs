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
    System::{SystemServices::MAXWORD, IO::OVERLAPPED},
};

use crate::{
    posix::types::*,
    posix::Errno,
    posix::MemZeroedStruct,
    posix::{
        shm_get_size, win32_security_attributes::from_security_attributes_to_mode, F_GETFD,
        F_GETFL, F_GETLK, F_RDLCK, F_SETFL, F_SETLK, F_SETLKW, F_UNLCK, F_WRLCK, O_NONBLOCK,
        S_IFDIR, S_IFREG,
    },
};

use super::{
    settings::MAX_PATH_LENGTH,
    win32_handle_translator::{FdHandleEntry, FileHandle, HandleTranslator},
    win32_security_attributes::from_mode_to_security_attributes,
};
use crate::posix;
use crate::win32call;

pub unsafe fn open_with_mode(pathname: *const c_char, flags: int, mode: mode_t) -> int {
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

    let (handle, _) = win32call! {CreateFileA(
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

    HandleTranslator::get_instance().add(FdHandleEntry::File(FileHandle {
        handle,
        lock_state: F_UNLCK,
    }))
}

pub unsafe fn open(pathname: *const c_char, flags: int) -> int {
    open_with_mode(pathname, flags, 0)
}

impl MemZeroedStruct for BY_HANDLE_FILE_INFORMATION {}

pub unsafe fn fstat(fd: int, buf: *mut stat_t) -> int {
    let mut file_stat = stat_t::new_zeroed();

    let permission_handle = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::SharedMemory(handle)) => {
            file_stat.st_size = shm_get_size(handle.state_handle) as _;
            handle.state_handle
        }
        Some(FdHandleEntry::File(handle)) => {
            let (size, _) = win32call! {GetFileSize(handle.handle, core::ptr::null_mut::<u32>())};
            if size == INVALID_FILE_SIZE {
                Errno::set(Errno::EINVAL);
                return -1;
            }

            file_stat.st_size = size as _;
            file_stat.st_mode = S_IFREG;

            let mut info = BY_HANDLE_FILE_INFORMATION::new_zeroed();
            let (has_file_info, _) =
                win32call! {GetFileInformationByHandle(handle.handle, &mut info)};
            if has_file_info == FALSE {
                Errno::set(Errno::EINVAL);
                return -1;
            }

            file_stat.st_mode = if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
                S_IFDIR
            } else {
                S_IFREG
            };

            handle.handle
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    };

    if let Some(file_path) = handle_to_file_path(permission_handle) {
        if let Some(mode) = acquire_mode_from_path(&file_path) {
            file_stat.st_mode |= mode;
        }
    };

    buf.write(file_stat);

    0
}

pub(crate) unsafe fn acquire_mode_from_path(file_path: &[u8]) -> Option<mode_t> {
    const SECURITY_BUFFER_CAPACITY: usize = 1024;
    let mut security_attributes_buffer = [0u8; SECURITY_BUFFER_CAPACITY];
    let mut required_buffer_capacity = 0;
    let (has_read_file_security, _) = win32call!(GetFileSecurityA(
        file_path.as_ptr(),
        DACL_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
        security_attributes_buffer.as_mut_ptr() as *mut void,
        SECURITY_BUFFER_CAPACITY as u32,
        &mut required_buffer_capacity
    ));
    if has_read_file_security == FALSE {
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
        Some(FdHandleEntry::Socket(mut socket)) => {
            if cmd == F_SETFL && (arg & O_NONBLOCK != 0) {
                socket.recv_timeout = None;
                HandleTranslator::get_instance().update(FdHandleEntry::Socket(socket));
            }
            socket.fd
        }
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

impl MemZeroedStruct for OVERLAPPED {}

pub unsafe fn fcntl(fd: int, cmd: int, arg: *mut flock) -> int {
    let handle = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::File(_)) => HandleTranslator::get_instance().get_file_handle_mut(fd),
        Some(FdHandleEntry::SharedMemory(_)) => {
            &mut HandleTranslator::get_instance()
                .get_shm_handle_mut(fd)
                .handle
        }
        _ => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    if cmd != F_SETLK && cmd != F_SETLKW && cmd != F_GETLK {
        Errno::set(Errno::EINVAL);
        return -1;
    }

    if cmd == F_GETLK {
        if handle.lock_state != posix::F_UNLCK {
            (*arg).l_type = handle.lock_state as i16;
            return 0;
        }

        let mut overlapped = OVERLAPPED::new_zeroed();
        let flags = LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY;
        let lock_result = LockFileEx(handle.handle, flags, 0, MAXWORD, MAXWORD, &mut overlapped);

        if lock_result != FALSE {
            let mut overlapped = OVERLAPPED::new_zeroed();
            UnlockFileEx(handle.handle, 0, MAXWORD, MAXWORD, &mut overlapped);
            (*arg).l_type = posix::F_UNLCK as _;
        } else {
            (*arg).l_type = posix::F_WRLCK as _;
        }

        return 0;
    }

    let lock_type = (*arg).l_type as i32;
    if lock_type != F_UNLCK && lock_type != F_RDLCK && lock_type != F_WRLCK {
        Errno::set(Errno::EINVAL);
        return -1;
    }

    if lock_type == F_UNLCK {
        let mut overlapped = OVERLAPPED::new_zeroed();
        handle.lock_state = F_UNLCK;
        let (file_unlocked, _) = win32call! {UnlockFileEx(
            handle.handle,
            0,
            MAXWORD,
            MAXWORD,
            &mut overlapped,
        )};
        if file_unlocked == FALSE {
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

    let mut overlapped = OVERLAPPED::new_zeroed();

    let (has_file_locked, _) =
        win32call! {LockFileEx(handle.handle, flags, 0, MAXWORD, MAXWORD, &mut overlapped)};
    if has_file_locked == FALSE {
        return -1;
    }

    handle.lock_state = lock_type;

    0
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
    let (file_path_len, _) = win32call!(GetFinalPathNameByHandleA(
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
    let handle = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::SharedMemory(handle)) => handle.state_handle,
        Some(FdHandleEntry::File(handle)) => handle.handle,
        _ => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    let file_path = match handle_to_file_path(handle) {
        Some(v) => v,
        None => {
            Errno::set(Errno::EINVAL);
            return -1;
        }
    };

    let security_attributes = from_mode_to_security_attributes(handle, mode);

    let (has_file_security_set, _) = win32call!(SetFileSecurityA(
        file_path.as_ptr(),
        DACL_SECURITY_INFORMATION,
        security_attributes.lpSecurityDescriptor
    ));
    if has_file_security_set == FALSE {
        Errno::set(Errno::EPERM);
        return -1;
    }

    0
}
