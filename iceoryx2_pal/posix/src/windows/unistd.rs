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
        CloseHandle, ERROR_FILE_NOT_FOUND, ERROR_NO_MORE_FILES, FALSE, INVALID_HANDLE_VALUE, TRUE,
    },
    Networking::WinSock::closesocket,
    Storage::FileSystem::{
        FlushFileBuffers, GetFileAttributesA, ReadFile, RemoveDirectoryA, SetFilePointer,
        WriteFile, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY, FILE_BEGIN, FILE_CURRENT,
        FILE_END, INVALID_FILE_ATTRIBUTES, INVALID_SET_FILE_POINTER,
    },
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        },
        SystemInformation::{GetSystemInfo, SYSTEM_INFO},
        Threading::GetCurrentProcessId,
        IO::OVERLAPPED,
    },
};

use crate::{
    posix::shm_set_size,
    posix::Struct,
    posix::{constants::*, types::*, win32_handle_translator::FdHandleEntry, Errno},
};

use super::{settings::MAX_PATH_LENGTH, win32_handle_translator::HandleTranslator};
use crate::win32call;

impl Struct for SYSTEM_INFO {}

pub unsafe fn sysconf(name: int) -> long {
    let mut system_info = SYSTEM_INFO::new();
    win32call! { GetSystemInfo(&mut system_info)};

    const POSIX_VERSION: long = 200809;

    match name {
        _SC_MONOTONIC_CLOCK => 0,
        _SC_PAGESIZE => system_info.dwPageSize as long,
        _SC_NPROCESSORS_CONF => system_info.dwNumberOfProcessors as long,
        _SC_VERSION => POSIX_VERSION,
        _SC_BARRIERS => POSIX_VERSION,
        _SC_MAPPED_FILES => POSIX_VERSION,
        _SC_READER_WRITER_LOCKS => POSIX_VERSION,
        _SC_SEMAPHORES => POSIX_VERSION,
        _SC_SHARED_MEMORY_OBJECTS => POSIX_VERSION,
        _SC_SPIN_LOCKS => POSIX_VERSION,
        _SC_TIMEOUTS => POSIX_VERSION,
        _SC_TIMERS => POSIX_VERSION,
        _SC_THREAD_SAFE_FUNCTIONS => POSIX_VERSION,
        _SC_SEM_VALUE_MAX => i32::MAX - 1,
        _SC_THREAD_STACK_MIN => 1024 * 1024,
        _SC_THREAD_THREADS_MAX => MAX_NUMBER_OF_THREADS as long,

        _ => {
            Errno::set(Errno::EINVAL);
            -1
        }
    }
}

pub unsafe fn pathconf(path: *const char, name: int) -> long {
    match name {
        _PC_PATH_MAX => MAX_PATH_LENGTH as long,
        _ => {
            Errno::set(Errno::EINVAL);
            -1
        }
    }
}

pub unsafe fn getpid() -> pid_t {
    win32call! { GetCurrentProcessId()  }
}

impl Struct for PROCESSENTRY32 {}

pub unsafe fn getppid() -> pid_t {
    let snapshot = win32call! { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return 0;
    }

    let mut process_entry = PROCESSENTRY32::new();
    process_entry.dwSize = core::mem::size_of::<PROCESSENTRY32>() as u32;

    let mut parent_process_id = 0;
    let self_process_id = getgid();

    if win32call! { Process32First(snapshot, &mut process_entry) } == TRUE {
        loop {
            if process_entry.th32ProcessID == self_process_id {
                parent_process_id = process_entry.th32ParentProcessID;
                break;
            }

            if win32call! { Process32Next(snapshot, &mut process_entry), ignore ERROR_NO_MORE_FILES }
                == FALSE
            {
                break;
            }
        }
    }

    parent_process_id
}

pub unsafe fn dup(fildes: int) -> int {
    -1
}

pub unsafe fn close(fd: int) -> int {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            win32call! { CloseHandle(handle.handle)};
            win32call! { CloseHandle(handle.state_handle)};
            HandleTranslator::get_instance().remove(fd);
            0
        }
        Some(FdHandleEntry::Socket(handle)) => {
            win32call! { winsock closesocket(handle.fd) };
            HandleTranslator::get_instance().remove(fd);
            0
        }
        Some(FdHandleEntry::UdsDatagramSocket(handle)) => {
            win32call! { winsock closesocket(handle.fd)};
            0
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn read(fd: int, buf: *mut void, count: size_t) -> ssize_t {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            let mut bytes_read = 0;
            if win32call! {ReadFile(
                handle.handle,
                buf,
                count as u32,
                &mut bytes_read,
                core::ptr::null_mut::<OVERLAPPED>(),
            )} == 0
            {
                -1
            } else {
                bytes_read as ssize_t
            }
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn write(fd: int, buf: *const void, count: size_t) -> ssize_t {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            let mut bytes_written = 0;
            if win32call! {WriteFile(
                handle.handle,
                buf as *const u8,
                count as u32,
                &mut bytes_written,
                core::ptr::null_mut::<OVERLAPPED>(),
            )} == 0
            {
                -1
            } else {
                bytes_written as ssize_t
            }
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn access(pathname: *const char, mode: int) -> int {
    let attributes =
        win32call! {GetFileAttributesA(pathname as *const u8), ignore ERROR_FILE_NOT_FOUND};

    if attributes == INVALID_FILE_ATTRIBUTES {
        if HandleTranslator::get_instance().contains_uds(pathname) {
            return 0;
        }
        -1
    } else {
        if mode == F_OK && attributes & FILE_ATTRIBUTE_DIRECTORY != 0 {
            Errno::set(Errno::ENOENT);
            return -1;
        }

        if mode == W_OK && attributes & FILE_ATTRIBUTE_READONLY != 0 {
            Errno::set(Errno::EACCES);
            return -1;
        }

        0
    }
}

pub unsafe fn unlink(pathname: *const char) -> int {
    -1
}

pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            let move_method = match whence {
                SEEK_SET => FILE_BEGIN,
                SEEK_CUR => FILE_CURRENT,
                SEEK_END => FILE_END,
                _ => {
                    return -1;
                }
            };

            let new_position = win32call! {SetFilePointer(handle.handle, offset, core::ptr::null_mut::<i32>(), move_method)};

            if new_position == INVALID_SET_FILE_POINTER {
                return -1;
            }

            new_position as off_t
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn getuid() -> uid_t {
    uid_t::MAX
}

pub unsafe fn getgid() -> gid_t {
    gid_t::MAX
}

pub unsafe fn rmdir(pathname: *const char) -> int {
    if win32call! {RemoveDirectoryA(pathname as*const u8), ignore ERROR_FILE_NOT_FOUND} == 0 {
        return -1;
    }
    0
}

pub unsafe fn ftruncate(fd: int, length: off_t) -> int {
    if length < 0 {
        Errno::set(Errno::EINVAL);
        return -1;
    }

    let win_handle = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(v)) => v,
        _ => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    shm_set_size(win_handle.state_handle, length as u64);
    0
}

pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> int {
    0
}

pub unsafe fn fsync(fd: int) -> int {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Handle(handle)) => {
            win32call! {FlushFileBuffers(handle.handle)};
            0
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}
