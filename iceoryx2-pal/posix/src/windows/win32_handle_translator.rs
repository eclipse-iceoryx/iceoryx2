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

use iceoryx2_pal_configuration::PATH_LENGTH;
use windows_sys::Win32::{
    Foundation::HANDLE,
    Networking::WinSock::SOCKADDR,
    System::Threading::{WaitOnAddress, WakeByAddressSingle, INFINITE},
};

use crate::posix::{c_string_length, types::*};
use core::sync::atomic::Ordering;
use core::{cell::UnsafeCell, panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU32, IoxAtomicUsize};
use iceoryx2_pal_concurrency_sync::mutex::Mutex;
use iceoryx2_pal_concurrency_sync::WaitAction;

use super::win32_udp_port_to_uds_name::PortToUds;

const MAX_SUPPORTED_FD_HANDLES: usize = 1024;

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FdHandleEntry {
    SharedMemory(ShmHandle),
    File(FileHandle),
    DirectoryStream(u64),
    Socket(SocketHandle),
    UdsDatagramSocket(UdsDatagramSocketHandle),
    NextFreeFd(usize),
}

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileHandle {
    pub handle: HANDLE,
    pub lock_state: int,
}

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShmHandle {
    pub handle: FileHandle,
    pub state_handle: HANDLE,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct UdsDatagramSocketHandle {
    pub fd: usize,
    pub address: Option<sockaddr_in>,
    pub recv_timeout: Option<timeval>,
}

impl PartialEq for UdsDatagramSocketHandle {
    fn eq(&self, other: &Self) -> bool {
        self.fd == other.fd
    }
}

impl Eq for UdsDatagramSocketHandle {}

impl UdsDatagramSocketHandle {
    pub fn address(&self) -> *const SOCKADDR {
        (self.address.as_ref().unwrap() as *const sockaddr_in) as *const SOCKADDR
    }

    pub fn address_len(&self) -> usize {
        core::mem::size_of::<sockaddr_in>()
    }

    pub fn port(&self) -> u16 {
        u16::from_be(self.address.as_ref().unwrap().sin_port)
    }

    pub fn is_set(&self) -> bool {
        self.address.is_some()
    }
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SocketHandle {
    pub fd: usize,
    pub recv_timeout: Option<timeval>,
    pub send_timeout: Option<timeval>,
}

impl PartialEq for SocketHandle {
    fn eq(&self, other: &Self) -> bool {
        self.fd == other.fd
    }
}

impl Eq for SocketHandle {}

#[doc(hidden)]
pub struct HandleTranslator {
    fd2handle: [UnsafeCell<FdHandleEntry>; MAX_SUPPORTED_FD_HANDLES],
    free_fd_list_start: UnsafeCell<usize>,
    port_to_uds_translator: UnsafeCell<Option<PortToUds>>,
    uds_datagram_counter: IoxAtomicUsize,
    mtx: Mutex,
}

unsafe impl Send for HandleTranslator {}
unsafe impl Sync for HandleTranslator {}

impl HandleTranslator {
    const fn new() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const NEXT_FREE_FD: UnsafeCell<FdHandleEntry> =
            UnsafeCell::new(FdHandleEntry::NextFreeFd(0));
        #[deny(clippy::declare_interior_mutable_const)]
        Self {
            fd2handle: [NEXT_FREE_FD; MAX_SUPPORTED_FD_HANDLES],
            free_fd_list_start: UnsafeCell::new(0),
            port_to_uds_translator: UnsafeCell::new(None),
            uds_datagram_counter: IoxAtomicUsize::new(0),
            mtx: Mutex::new(),
        }
    }

    pub fn get_instance() -> &'static HandleTranslator {
        static HANDLE_TRANSLATOR: HandleTranslator = HandleTranslator::new();
        static IS_INITIALIZED: IoxAtomicBool = IoxAtomicBool::new(false);

        if !IS_INITIALIZED.load(Ordering::Relaxed) {
            HANDLE_TRANSLATOR.lock();
            // double check to avoid unnecessary locks
            if !IS_INITIALIZED.load(Ordering::Relaxed) {
                for i in 0..MAX_SUPPORTED_FD_HANDLES {
                    unsafe {
                        *HANDLE_TRANSLATOR.fd2handle[i].get() = FdHandleEntry::NextFreeFd(i + 1);
                    }
                }
                IS_INITIALIZED.store(true, Ordering::Relaxed);
            }
            HANDLE_TRANSLATOR.unlock();
        }

        &HANDLE_TRANSLATOR
    }

    fn lock(&self) {
        self.mtx.lock(|atomic, value| {
            unsafe {
                WaitOnAddress(
                    (atomic as *const IoxAtomicU32).cast(),
                    (value as *const u32).cast(),
                    4,
                    INFINITE,
                );
            }
            WaitAction::Continue
        });
    }

    fn unlock(&self) {
        self.mtx.unlock(|atomic| unsafe {
            WakeByAddressSingle((atomic as *const IoxAtomicU32).cast());
        });
    }

    pub fn add(&self, entry: FdHandleEntry) -> int {
        self.lock();
        let free_fd_list_start = unsafe { *self.free_fd_list_start.get() };
        if free_fd_list_start >= MAX_SUPPORTED_FD_HANDLES {
            self.unlock();
            return -1;
        }

        let next_free_fd = match unsafe { *self.fd2handle[free_fd_list_start].get() } {
            FdHandleEntry::NextFreeFd(fd) => fd,
            _ => {
                self.unlock();
                panic!("This should never happen! Corrupted HandleTranslator::add.")
            }
        };

        unsafe {
            *self.fd2handle[free_fd_list_start].get() = entry;
            *self.free_fd_list_start.get() = next_free_fd;
        }

        if let FdHandleEntry::UdsDatagramSocket(_) = entry {
            if self.uds_datagram_counter.fetch_add(1, Ordering::Relaxed) == 0 {
                unsafe {
                    *self.port_to_uds_translator.get() = Some(
                        PortToUds::new()
                            .expect("Unable to create port to uds datagram name translator."),
                    )
                };
            }
        }

        self.unlock();

        free_fd_list_start as _
    }

    pub(crate) fn get(&self, fd: int) -> Option<FdHandleEntry> {
        let mut ret_val = None;
        self.lock();
        if (fd as usize) < self.fd2handle.len() {
            ret_val = Some(unsafe { *self.fd2handle[fd as usize].get() });
            if let Some(FdHandleEntry::NextFreeFd(_)) = ret_val {
                ret_val = None;
            }
        }
        self.unlock();
        ret_val
    }

    pub(crate) fn update(&self, entry: FdHandleEntry) {
        self.lock();
        for i in 0..MAX_SUPPORTED_FD_HANDLES {
            if unsafe { *self.fd2handle[i].get() == entry } {
                unsafe { *self.fd2handle[i].get() = entry };
                break;
            }
        }
        self.unlock();
    }

    pub(crate) fn get_fd(&self, entry: FdHandleEntry) -> int {
        self.lock();
        for i in 0..MAX_SUPPORTED_FD_HANDLES {
            if unsafe { *self.fd2handle[i].get() == entry } {
                self.unlock();
                return i as int;
            }
        }
        self.unlock();
        -1
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_shm_handle_mut(&self, fd: int) -> &mut ShmHandle {
        self.lock();
        match unsafe { &mut *self.fd2handle[fd as usize].get() } {
            FdHandleEntry::SharedMemory(ref mut handle) => {
                self.unlock();
                handle
            }
            _ => {
                self.unlock();
                panic!("Accessed invalid file descriptor.");
            }
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_file_handle_mut(&self, fd: int) -> &mut FileHandle {
        self.lock();
        match unsafe { &mut *self.fd2handle[fd as usize].get() } {
            FdHandleEntry::File(ref mut handle) => {
                self.unlock();
                handle
            }
            _ => {
                self.unlock();
                panic!("Accessed invalid file descriptor.");
            }
        }
    }

    pub(crate) unsafe fn get_socket(&self, fd: int) -> Option<SocketHandle> {
        self.lock();
        match unsafe { *self.fd2handle[fd as usize].get() } {
            FdHandleEntry::Socket(handle) => {
                self.unlock();
                Some(handle)
            }
            _ => {
                self.unlock();
                None
            }
        }
    }

    pub(crate) fn remove(&self, fd: int) {
        self.lock();
        self.remove_impl(fd);
        self.unlock();
    }

    pub(crate) fn remove_entry(&self, entry: FdHandleEntry) {
        self.lock();
        for i in 0..MAX_SUPPORTED_FD_HANDLES {
            if unsafe { *self.fd2handle[i].get() == entry } {
                self.remove_impl(i as _);
                break;
            }
        }
        self.unlock();
    }

    pub(crate) fn contains_uds(&self, name: *const c_char) -> bool {
        let mut result = false;
        let name_slice = unsafe { core::slice::from_raw_parts(name.cast(), c_string_length(name)) };

        self.lock();
        if let Some(ref v) = unsafe { &(*self.port_to_uds_translator.get()) } {
            result = v.contains(name_slice);
        }
        self.unlock();
        result
    }

    pub(crate) fn remove_uds(&self, name: *const c_char) -> bool {
        let mut result = false;
        let name_slice = unsafe { core::slice::from_raw_parts(name.cast(), c_string_length(name)) };

        self.lock();
        if let Some(ref v) = unsafe { &(*self.port_to_uds_translator.get()) } {
            result = v.remove(name_slice);
        }
        self.unlock();
        result
    }

    pub(crate) fn list_all_uds(&self, path: *const c_char) -> Vec<[u8; PATH_LENGTH]> {
        let mut result = vec![];
        let path_slice = unsafe { core::slice::from_raw_parts(path.cast(), c_string_length(path)) };

        self.lock();
        if let Some(ref v) = unsafe { &(*self.port_to_uds_translator.get()) } {
            result = v.list(path_slice);
        }
        self.unlock();

        result
    }

    pub(crate) fn set_uds_name(&self, fd: int, address: sockaddr_in, uds_address: *const sockaddr) {
        let port = u16::from_be(address.sin_port);

        let uds_address = uds_address as *const sockaddr_un;
        let name = unsafe {
            core::slice::from_raw_parts(
                (*uds_address).sun_path.as_ptr() as *const u8,
                (*uds_address).sun_path.len(),
            )
        };

        unsafe {
            self.lock();
            if let FdHandleEntry::UdsDatagramSocket(ref mut s) = *self.fd2handle[fd as usize].get()
            {
                s.address = Some(address);
            }

            (*self.port_to_uds_translator.get())
                .as_ref()
                .unwrap()
                .set(port, name);
            self.unlock();
        };
    }

    pub(crate) fn get_uds_port(&self, uds_address: *const sockaddr) -> u16 {
        let uds_address = uds_address as *const sockaddr_un;
        let name = unsafe {
            core::slice::from_raw_parts(
                (*uds_address).sun_path.as_ptr() as *const u8,
                (*uds_address).sun_path.len(),
            )
        };

        self.lock();
        let result = unsafe {
            (*self.port_to_uds_translator.get())
                .as_ref()
                .unwrap()
                .get_port(name)
        };
        self.unlock();
        result
    }

    fn remove_impl(&self, fd: int) {
        unsafe {
            if let FdHandleEntry::UdsDatagramSocket(s) = *self.fd2handle[fd as usize].get() {
                if self.uds_datagram_counter.fetch_sub(1, Ordering::Relaxed) == 1 {
                    *self.port_to_uds_translator.get() = None;
                } else if s.is_set() {
                    (*self.port_to_uds_translator.get())
                        .as_ref()
                        .unwrap()
                        .reset(s.port())
                }
            }

            *self.fd2handle[fd as usize].get() =
                FdHandleEntry::NextFreeFd(*self.free_fd_list_start.get());
            *self.free_fd_list_start.get() = fd as usize;
        }
    }
}
