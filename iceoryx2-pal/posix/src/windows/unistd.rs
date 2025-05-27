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
    Networking::WinSock::{
        closesocket, WSADuplicateSocketA, WSASocketA, INVALID_SOCKET, SOCKET_ERROR,
        WSAPROTOCOL_INFOA,
    },
    Storage::FileSystem::{
        FlushFileBuffers, GetFileAttributesA, ReadFile, RemoveDirectoryA, SetEndOfFile,
        SetFilePointerEx, WriteFile, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_READONLY, FILE_BEGIN,
        FILE_CURRENT, FILE_END, INVALID_FILE_ATTRIBUTES,
    },
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32,
            TH32CS_SNAPPROCESS,
        },
        ProcessStatus::GetModuleFileNameExA,
        SystemInformation::{GetSystemInfo, SYSTEM_INFO},
        Threading::{GetCurrentProcessId, OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
        IO::OVERLAPPED,
    },
};

use crate::{
    posix::shm_set_size,
    posix::MemZeroedStruct,
    posix::{constants::*, types::*, win32_handle_translator::FdHandleEntry, Errno},
};

use super::{
    settings::MAX_PATH_LENGTH,
    win32_handle_translator::{HandleTranslator, SocketHandle},
};
use crate::win32call;

impl MemZeroedStruct for SYSTEM_INFO {}

pub unsafe fn proc_pidpath(pid: pid_t, buffer: *mut c_char, buffer_len: size_t) -> isize {
    let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, pid as _);

    if process_handle == 0 {
        return -1;
    }

    let ret_val = GetModuleFileNameExA(process_handle, 0, buffer.cast(), buffer_len as _);
    CloseHandle(process_handle);

    if ret_val == 0 {
        return -1;
    }

    ret_val as _
}

pub unsafe fn sysconf(name: int) -> long {
    let mut system_info = SYSTEM_INFO::new_zeroed();
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

pub unsafe fn pathconf(path: *const c_char, name: int) -> long {
    match name {
        _PC_PATH_MAX => MAX_PATH_LENGTH as long,
        _ => {
            Errno::set(Errno::EINVAL);
            -1
        }
    }
}

pub unsafe fn getpid() -> pid_t {
    let (pid, _) = win32call! { GetCurrentProcessId() };
    pid
}

impl MemZeroedStruct for PROCESSENTRY32 {}

pub unsafe fn getppid() -> pid_t {
    let (snapshot, _) = win32call! { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return 0;
    }

    let mut process_entry = PROCESSENTRY32::new_zeroed();
    process_entry.dwSize = core::mem::size_of::<PROCESSENTRY32>() as u32;

    let mut parent_process_id = 0;
    let self_process_id = getgid();

    let (has_snapshot, _) = win32call! { Process32First(snapshot, &mut process_entry) };
    if has_snapshot == TRUE {
        loop {
            if process_entry.th32ProcessID == self_process_id {
                parent_process_id = process_entry.th32ParentProcessID;
                break;
            }

            let (has_snapshot, _) = win32call! { Process32Next(snapshot, &mut process_entry), ignore ERROR_NO_MORE_FILES };
            if has_snapshot == FALSE {
                break;
            }
        }
    }

    parent_process_id
}

impl MemZeroedStruct for WSAPROTOCOL_INFOA {}

pub unsafe fn dup(fildes: int) -> int {
    match HandleTranslator::get_instance().get(fildes) {
        Some(FdHandleEntry::Socket(handle)) => {
            let mut protocol_info = WSAPROTOCOL_INFOA::new_zeroed();
            let (result, _) = win32call! { winsock WSADuplicateSocketA(handle.fd, GetCurrentProcessId(), &mut protocol_info) };
            if result == SOCKET_ERROR {
                return -1;
            }
            let (duplicated_socket, _) =
                win32call! { WSASocketA(AF_UNIX as _, SOCK_STREAM, 0, &protocol_info, 0, 0)};
            if duplicated_socket == INVALID_SOCKET {
                return -1;
            }

            HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle {
                fd: duplicated_socket,
                recv_timeout: None,
                send_timeout: None,
            }))
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn close(fd: int) -> int {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::SharedMemory(handle)) => {
            HandleTranslator::get_instance().remove(fd);
            win32call! { CloseHandle(handle.handle.handle)};
            win32call! { CloseHandle(handle.state_handle)};
            0
        }
        Some(FdHandleEntry::File(handle)) => {
            HandleTranslator::get_instance().remove(fd);
            win32call! { CloseHandle(handle.handle)};
            0
        }
        Some(FdHandleEntry::Socket(handle)) => {
            HandleTranslator::get_instance().remove(fd);
            win32call! { winsock closesocket(handle.fd) };
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
        Some(FdHandleEntry::File(handle)) => {
            let mut bytes_read = 0;
            let (file_read, _) = win32call! {ReadFile(
                handle.handle,
                buf,
                count as u32,
                &mut bytes_read,
                core::ptr::null_mut::<OVERLAPPED>(),
            )};
            if file_read == FALSE {
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
        Some(FdHandleEntry::File(handle)) => {
            let mut bytes_written = 0;
            let (file_written, _) = win32call! {WriteFile(
                handle.handle,
                buf as *const u8,
                count as u32,
                &mut bytes_written,
                core::ptr::null_mut::<OVERLAPPED>(),
            )};
            if file_written == FALSE {
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

pub unsafe fn access(pathname: *const c_char, mode: int) -> int {
    let (attributes, _) =
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

pub unsafe fn unlink(pathname: *const c_char) -> int {
    -1
}

pub unsafe fn lseek(fd: int, offset: off_t, whence: int) -> off_t {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::File(handle)) => {
            let move_method = match whence {
                SEEK_SET => FILE_BEGIN,
                SEEK_CUR => FILE_CURRENT,
                SEEK_END => FILE_END,
                _ => {
                    return -1;
                }
            };

            let mut new_position = 0;
            let (has_success, _) = win32call! {SetFilePointerEx(handle.handle, offset, &mut new_position, move_method)};

            if has_success == 0 {
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

pub unsafe fn rmdir(pathname: *const c_char) -> int {
    let (has_removed, _) =
        win32call! {RemoveDirectoryA(pathname as*const u8), ignore ERROR_FILE_NOT_FOUND};
    if has_removed == FALSE {
        return -1;
    }
    0
}

pub unsafe fn ftruncate(fd: int, length: off_t) -> int {
    if length < 0 {
        Errno::set(Errno::EINVAL);
        return -1;
    }

    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::SharedMemory(handle)) => {
            shm_set_size(handle.state_handle, length as u64);
            0
        }
        Some(FdHandleEntry::File(handle)) => {
            let (result, _) = win32call! { SetFilePointerEx(
                handle.handle,
                length as _,
                core::ptr::null_mut(),
                FILE_BEGIN,
            )};
            if result == 0 {
                return -1;
            }
            win32call! { SetEndOfFile(handle.handle)};
            0
        }
        _ => {
            Errno::set(Errno::EBADF);
            0
        }
    }
}

pub unsafe fn fchown(fd: int, owner: uid_t, group: gid_t) -> int {
    0
}

pub unsafe fn fsync(fd: int) -> int {
    match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::File(handle)) => {
            win32call! {FlushFileBuffers(handle.handle)};
            0
        }
        _ => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}
