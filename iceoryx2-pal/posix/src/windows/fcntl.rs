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
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_FILE_EXISTS, ERROR_FILE_NOT_FOUND,
        ERROR_FILE_TOO_LARGE, ERROR_INVALID_HANDLE, ERROR_PATH_NOT_FOUND, ERROR_SUCCESS, FALSE,
        GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    },
    Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT, SetSecurityInfo},
    Security::{
        ACL, DACL_SECURITY_INFORMATION, GROUP_SECURITY_INFORMATION, GetFileSecurityA,
        GetSecurityDescriptorDacl, OWNER_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
        PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
    },
    Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, CREATE_NEW, CreateFileA, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_NORMAL, FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE,
        GetFileInformationByHandle, GetFileSize, INVALID_FILE_SIZE, LOCKFILE_EXCLUSIVE_LOCK,
        LOCKFILE_FAIL_IMMEDIATELY, LockFileEx, OPEN_ALWAYS, OPEN_EXISTING, ReOpenFile,
        UnlockFileEx, WRITE_DAC,
    },
    System::{IO::OVERLAPPED, Memory::LocalFree, SystemServices::MAXWORD},
};

use crate::{
    posix::Errno,
    posix::MemZeroedStruct,
    posix::types::*,
    posix::{
        F_GETFD, F_GETFL, F_GETLK, F_RDLCK, F_SETFL, F_SETLK, F_SETLKW, F_UNLCK, F_WRLCK,
        O_NONBLOCK, S_IFDIR, S_IFREG, shm_get_size,
        win32_security_attributes::from_security_attributes_to_mode,
    },
};

use super::{
    win32_handle_translator::{FdHandleEntry, FileHandle, HandleTranslator},
    win32_security_attributes::from_mode_to_security_attributes,
};
use crate::posix;
use crate::win32call;

pub unsafe fn open_with_mode(pathname: *const c_char, flags: int, mode: mode_t) -> int {
    let mut access_mode = if flags & posix::O_RDONLY != 0 {
        GENERIC_READ
    } else if flags & posix::O_WRONLY != 0 {
        GENERIC_WRITE
    } else if flags & posix::O_RDWR != 0 {
        GENERIC_WRITE | GENERIC_READ
    } else {
        0
    };
    if flags & posix::O_CREAT != 0 && flags & posix::O_EXCL != 0 {
        access_mode |= WRITE_DAC;
    }

    let shared_mode = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    let open_mode = if flags & posix::O_CREAT != 0 && flags & posix::O_EXCL != 0 {
        CREATE_NEW
    } else if flags & posix::O_CREAT != 0 {
        OPEN_ALWAYS
    } else {
        OPEN_EXISTING
    };

    let security_attributes = from_mode_to_security_attributes(mode);
    unsafe {
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
}

pub unsafe fn open(pathname: *const c_char, flags: int) -> int {
    unsafe { open_with_mode(pathname, flags, 0) }
}

impl MemZeroedStruct for BY_HANDLE_FILE_INFORMATION {}

pub unsafe fn fstat(fd: int, buf: *mut stat_t) -> int {
    let mut file_stat = stat_t::new_zeroed();
    unsafe {
        let permission_handle = match HandleTranslator::get_instance().get(fd) {
            Some(FdHandleEntry::SharedMemory(handle)) => {
                file_stat.st_size = shm_get_size(handle.state_handle) as _;
                handle.state_handle
            }
            Some(FdHandleEntry::File(handle)) => {
                let (size, _) =
                    win32call! {GetFileSize(handle.handle, core::ptr::null_mut::<u32>())};
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

                file_stat.st_nlink = info.nNumberOfLinks as _;

                handle.handle
            }
            _ => {
                Errno::set(Errno::EBADF);
                return -1;
            }
        };

        let mode = match acquire_mode_from_handle(permission_handle) {
            Ok(mode) => mode,
            Err(error) => {
                Errno::set(error);
                return -1;
            }
        };
        file_stat.st_mode |= mode;

        buf.write(file_stat);

        0
    }
}

unsafe fn acquire_mode_from_handle(handle: HANDLE) -> Result<mode_t, Errno> {
    // INVALID_HANDLE_VALUE also has the value of a Windows pseudo-handle. Reject it before
    // asking the security API to interpret it as an object handle.
    if handle == 0 || handle == INVALID_HANDLE_VALUE {
        return Err(Errno::EBADF);
    }
    let mut security_descriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut::<void>();
    let result = unsafe {
        GetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            core::ptr::null_mut::<*mut void>(),
            core::ptr::null_mut::<*mut void>(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut security_descriptor,
        )
    };
    if result != ERROR_SUCCESS {
        return Err(match result {
            ERROR_ACCESS_DENIED => Errno::EACCES,
            ERROR_INVALID_HANDLE => Errno::EBADF,
            _ => Errno::EIO,
        });
    }

    let security_attributes = SECURITY_ATTRIBUTES {
        nLength: core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: security_descriptor,
        bInheritHandle: FALSE,
    };
    let mode = from_security_attributes_to_mode(&security_attributes);
    unsafe { LocalFree(security_descriptor as isize) };

    Ok(mode)
}

pub(crate) unsafe fn acquire_mode_from_path(file_path: &[u8]) -> Option<mode_t> {
    const SECURITY_BUFFER_CAPACITY: usize = 1024;
    let mut security_attributes_buffer = [0u8; SECURITY_BUFFER_CAPACITY];
    let mut required_buffer_capacity = 0;
    unsafe {
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
        unsafe {
            win32call!(winsock windows_sys::Win32::Networking::WinSock::ioctlsocket(socket_fd, windows_sys::Win32::Networking::WinSock::FIONBIO, &mut mode));
        }
        0
    } else {
        Errno::set(Errno::ENOTSUP);
        -1
    }
}

impl MemZeroedStruct for OVERLAPPED {}

unsafe fn probe_file_lock(handle: HANDLE, requested: i16) -> Result<i16, Errno> {
    use windows_sys::Win32::Foundation::{ERROR_LOCK_VIOLATION, GetLastError};
    let flags = LOCKFILE_FAIL_IMMEDIATELY
        | if requested as i32 == F_RDLCK {
            0
        } else {
            LOCKFILE_EXCLUSIVE_LOCK
        };
    let mut overlapped = OVERLAPPED::new_zeroed();
    if unsafe { LockFileEx(handle, flags, 0, MAXWORD, MAXWORD, &mut overlapped) } == FALSE {
        return match unsafe { GetLastError() } {
            ERROR_LOCK_VIOLATION => Ok(F_WRLCK as _),
            ERROR_ACCESS_DENIED => Err(Errno::EACCES),
            ERROR_INVALID_HANDLE => Err(Errno::EBADF),
            _ => Err(Errno::EIO),
        };
    }
    let mut overlapped = OVERLAPPED::new_zeroed();
    if unsafe { UnlockFileEx(handle, 0, MAXWORD, MAXWORD, &mut overlapped) } == FALSE {
        return Err(match unsafe { GetLastError() } {
            ERROR_ACCESS_DENIED => Errno::EACCES,
            ERROR_INVALID_HANDLE => Errno::EBADF,
            _ => Errno::EIO,
        });
    }
    Ok(F_UNLCK as _)
}

pub unsafe fn fcntl(fd: int, cmd: int, arg: *mut flock) -> int {
    unsafe {
        let handle = match HandleTranslator::get_instance().get(fd) {
            Some(FdHandleEntry::File(_)) => {
                HandleTranslator::get_instance().get_file_handle_mut(fd)
            }
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

            match probe_file_lock(handle.handle, (*arg).l_type) {
                Ok(lock_type) => (*arg).l_type = lock_type,
                Err(error) => {
                    Errno::set(error);
                    return -1;
                }
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

pub unsafe fn fchmod(fd: int, mode: mode_t) -> int {
    let handle = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::SharedMemory(handle)) => handle.state_handle,
        Some(FdHandleEntry::File(handle)) => handle.handle,
        Some(FdHandleEntry::DirectoryStream(handle)) => return 0,
        e => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    let security_attributes = from_mode_to_security_attributes(mode);
    let mut dacl_present = FALSE;
    let mut dacl_defaulted = FALSE;
    let mut dacl = core::ptr::null_mut::<ACL>();
    let (has_dacl, _) = unsafe {
        win32call!(GetSecurityDescriptorDacl(
            security_attributes.lpSecurityDescriptor,
            &mut dacl_present,
            &mut dacl,
            &mut dacl_defaulted,
        ))
    };
    if has_dacl == FALSE || dacl_present == FALSE {
        Errno::set(Errno::EIO);
        return -1;
    }

    let mut result = unsafe {
        SetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            dacl,
            core::ptr::null_mut(),
        )
    };

    if result == ERROR_ACCESS_DENIED {
        let (permission_handle, reopen_error) = unsafe {
            win32call!(ReOpenFile(
                handle,
                WRITE_DAC | windows_sys::Win32::Storage::FileSystem::READ_CONTROL,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                0,
            ))
        };
        if permission_handle != INVALID_HANDLE_VALUE {
            result = unsafe {
                SetSecurityInfo(
                    permission_handle,
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    dacl,
                    core::ptr::null_mut(),
                )
            };
            unsafe { CloseHandle(permission_handle) };
        } else {
            result = reopen_error;
        }
    }

    if result == ERROR_SUCCESS {
        return 0;
    }

    Errno::set(match result {
        ERROR_ACCESS_DENIED => Errno::EACCES,
        ERROR_INVALID_HANDLE => Errno::EBADF,
        _ => Errno::EIO,
    });
    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_lock_probe_is_an_error_not_a_lock() {
        assert!(matches!(
            unsafe { probe_file_lock(INVALID_HANDLE_VALUE, F_RDLCK as _) },
            Err(Errno::EBADF)
        ));
    }

    #[test]
    fn shared_probes_coexist_and_release_their_locks() {
        use std::os::windows::io::AsRawHandle;
        let path = std::env::temp_dir().join(format!(
            "iox2_probe_{}_{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let owner = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        let observer = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let mut overlapped = OVERLAPPED::new_zeroed();
        assert_ne!(
            unsafe {
                LockFileEx(
                    owner.as_raw_handle() as _,
                    LOCKFILE_FAIL_IMMEDIATELY,
                    0,
                    MAXWORD,
                    MAXWORD,
                    &mut overlapped,
                )
            },
            FALSE
        );
        assert_eq!(
            unsafe { probe_file_lock(observer.as_raw_handle() as _, F_RDLCK as _) }.unwrap(),
            F_UNLCK as i16
        );
        assert_eq!(
            unsafe { probe_file_lock(observer.as_raw_handle() as _, F_WRLCK as _) }.unwrap(),
            F_WRLCK as i16
        );
        assert_ne!(
            unsafe {
                UnlockFileEx(
                    owner.as_raw_handle() as _,
                    0,
                    MAXWORD,
                    MAXWORD,
                    &mut overlapped,
                )
            },
            FALSE
        );
        assert_eq!(
            unsafe { probe_file_lock(observer.as_raw_handle() as _, F_WRLCK as _) }.unwrap(),
            F_UNLCK as i16
        );
        assert_ne!(
            unsafe {
                LockFileEx(
                    owner.as_raw_handle() as _,
                    LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                    0,
                    MAXWORD,
                    MAXWORD,
                    &mut overlapped,
                )
            },
            FALSE
        );
        assert_eq!(
            unsafe { probe_file_lock(observer.as_raw_handle() as _, F_RDLCK as _) }.unwrap(),
            F_WRLCK as i16
        );
        assert_ne!(
            unsafe {
                UnlockFileEx(
                    owner.as_raw_handle() as _,
                    0,
                    MAXWORD,
                    MAXWORD,
                    &mut overlapped,
                )
            },
            FALSE
        );
        drop(observer);
        drop(owner);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn failed_security_lookup_does_not_return_empty_permissions() {
        assert!(matches!(
            unsafe { acquire_mode_from_handle(INVALID_HANDLE_VALUE) },
            Err(Errno::EBADF)
        ));
    }

    #[test]
    fn invalid_descriptor_does_not_publish_metadata() {
        let mut metadata = stat_t::new_zeroed();
        metadata.st_mode = posix::S_IRUSR;
        assert_eq!(unsafe { fstat(-1, &mut metadata) }, -1);
        assert_eq!(metadata.st_mode, posix::S_IRUSR);
    }
}
