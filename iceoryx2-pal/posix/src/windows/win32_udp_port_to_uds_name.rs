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
    Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, HANDLE},
    Security::SECURITY_ATTRIBUTES,
    System::Memory::{
        CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, VirtualAlloc, FILE_MAP_ALL_ACCESS,
        MEM_COMMIT, PAGE_READWRITE, SEC_RESERVE,
    },
};

use crate::posix::{c_string_length, types::*};
use core::ffi::CStr;
use core::{cell::UnsafeCell, sync::atomic::Ordering};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

const IS_INITIALIZED: u64 = 0xaffedeadbeef;
const INITIALIZATION_IN_PROGRESS: u64 = 0xbebebebebebebebe;
const SHM_SEGMENT_NAME: &CStr = c"/port_to_uds_name_map";
const SHM_SIZE: usize = core::mem::size_of::<PortToUdsNameMap>();
const UNINITIALIZED_ENTRY: u64 = 1;

struct Entries<const N: usize> {
    aba_counters: [IoxAtomicU64; N],
    storages: [[UnsafeCell<[u8; PATH_LENGTH]>; 2]; N],
}

impl<const N: usize> Entries<N> {
    fn initialize(&mut self) {
        for i in 0..N {
            self.aba_counters[i] = IoxAtomicU64::new(UNINITIALIZED_ENTRY);
        }
    }

    fn is_set(&self, index: usize) -> bool {
        let aba_counter = &self.aba_counters[index];
        let storage = &self.storages[index];

        let mut current = aba_counter.load(Ordering::Relaxed);
        if current == UNINITIALIZED_ENTRY {
            return false;
        }

        loop {
            let first_char = unsafe { (*storage[((current - 1) % 2) as usize].get())[0] };
            if current != aba_counter.load(Ordering::Relaxed) {
                current = aba_counter.load(Ordering::Relaxed);
                continue;
            }

            return first_char != 0;
        }
    }

    fn set(&self, index: usize, value: &[u8]) {
        let aba_counter = &self.aba_counters[index];
        let storage = &self.storages[index];

        let current = aba_counter.load(Ordering::Acquire);
        unsafe {
            (*storage[(current % 2) as usize].get()) = [0u8; PATH_LENGTH];
            (&mut (*storage[(current % 2) as usize].get()))[..value.len()].copy_from_slice(value);
        };
        aba_counter.fetch_add(1, Ordering::Release);
    }

    fn get(&self, index: usize) -> Option<[u8; PATH_LENGTH]> {
        let aba_counter = &self.aba_counters[index];
        let storage = &self.storages[index];

        let current = aba_counter.load(Ordering::Acquire);
        let mut result;
        loop {
            if current == UNINITIALIZED_ENTRY {
                return None;
            }

            result = unsafe { *storage[((current - 1) % 2) as usize].get() };
            if current == aba_counter.load(Ordering::Acquire) {
                break;
            }
        }

        Some(result)
    }

    fn reset(&self, index: usize) {
        self.aba_counters[index].store(UNINITIALIZED_ENTRY, Ordering::Relaxed);
    }

    fn list(&self) -> Vec<[u8; PATH_LENGTH]> {
        let mut result = vec![];

        for i in 0..N {
            if !self.is_set(i) {
                continue;
            }

            result.push(self.get(i).unwrap());
        }

        result
    }

    const fn len(&self) -> usize {
        N
    }
}

fn normalized_name(name: &[u8]) -> [u8; PATH_LENGTH] {
    let mut result = [0u8; PATH_LENGTH];
    let low_case = name.to_ascii_lowercase();
    let mut is_previous_char_path_separator = false;

    let mut n = 0;
    for c in &low_case {
        if *c == b'\\' || *c == b'/' {
            if is_previous_char_path_separator {
                continue;
            }
            is_previous_char_path_separator = true;
        } else {
            is_previous_char_path_separator = false;
        }

        result[n] = *c;
        n += 1;
    }

    result
}

#[repr(C)]
struct PortToUdsNameMap {
    init_check: IoxAtomicU64,
    uds_names: Entries<65535>,
}

impl PortToUdsNameMap {
    fn initialize(&mut self) {
        let current = self.init_check.load(Ordering::Relaxed);

        match self.init_check.compare_exchange(
            current,
            INITIALIZATION_IN_PROGRESS,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                self.uds_names.initialize();
                self.init_check.store(IS_INITIALIZED, Ordering::Relaxed);
            }
            Err(_) => while self.init_check.load(Ordering::Relaxed) != IS_INITIALIZED {},
        }
    }

    fn set(&self, port: u16, name: &[u8]) {
        let name = normalized_name(name);
        self.uds_names.set(port as usize, &name);
    }

    fn get_port(&self, name: &[u8]) -> u16 {
        let name = normalized_name(name);
        for i in 0..self.uds_names.len() {
            match self.uds_names.get(i) {
                Some(entry_name) => {
                    let pos = entry_name.iter().position(|c| *c == 0).unwrap_or(0);
                    if pos == 0 || pos > name.len() {
                        continue;
                    }

                    if entry_name[..pos] == name[..pos] {
                        return i as _;
                    }
                }
                None => continue,
            }
        }
        0
    }

    fn contains(&self, name: &[u8]) -> bool {
        self.get_port(name) != 0
    }

    fn remove(&self, name: &[u8]) -> bool {
        let port = self.get_port(name);
        if port == 0 {
            return false;
        }

        self.uds_names.reset(port as usize);
        true
    }
}

#[doc(hidden)]
pub struct PortToUds {
    shm_handle: HANDLE,
    map: *const PortToUdsNameMap,
}

unsafe impl Send for PortToUds {}
unsafe impl Sync for PortToUds {}

impl Drop for PortToUds {
    fn drop(&mut self) {
        unsafe {
            win32call! { UnmapViewOfFile(self.map as isize)};
            win32call! { CloseHandle(self.shm_handle)};
        }
    }
}

impl PortToUds {
    pub fn new() -> Option<Self> {
        let handle: HANDLE = 0;
        let (shm_handle, last_error) = unsafe {
            win32call! { CreateFileMappingA(handle, core::ptr::null::<SECURITY_ATTRIBUTES>(), PAGE_READWRITE | SEC_RESERVE, 0, SHM_SIZE as _, SHM_SEGMENT_NAME.as_ptr() as *const u8), ignore ERROR_ALREADY_EXISTS}
        };

        if shm_handle == 0 {
            return None;
        }

        let has_created_shm = last_error != ERROR_ALREADY_EXISTS;

        let (map_result, _) = unsafe {
            win32call! {MapViewOfFile(shm_handle, FILE_MAP_ALL_ACCESS, 0, 0, SHM_SIZE as _)}
        };

        if map_result == 0 {
            unsafe {
                win32call! { CloseHandle(shm_handle) }
            };
            return None;
        }

        let (base_address, _) = unsafe {
            win32call! {VirtualAlloc(map_result as *const void, SHM_SIZE as _, MEM_COMMIT, PAGE_READWRITE)}
        };
        if base_address.is_null() {
            unsafe {
                win32call! { UnmapViewOfFile(map_result)};
                win32call! { CloseHandle(shm_handle)};
            }
            return None;
        }

        let map = map_result as *mut PortToUdsNameMap;

        if has_created_shm {
            unsafe { &mut *map }.initialize();
        }

        Some(Self { shm_handle, map })
    }

    pub fn set(&self, port: u16, name: &[u8]) {
        unsafe { (*self.map).set(port, name) }
    }

    pub fn contains(&self, name: &[u8]) -> bool {
        unsafe { (*self.map).contains(name) }
    }

    pub fn remove(&self, name: &[u8]) -> bool {
        unsafe { (*self.map).remove(name) }
    }

    pub fn list(&self, path: &[u8]) -> Vec<[u8; PATH_LENGTH]> {
        let path = normalized_name(path);
        let path_len = unsafe { c_string_length(path.as_ptr().cast()) };

        let mut result = vec![];

        for value in unsafe { &(*self.map).uds_names.list() } {
            let value_len = unsafe { c_string_length(value.as_ptr().cast()) };
            if value_len == 0 || value_len <= path_len {
                continue;
            }

            if value[..path_len] == path[..path_len] {
                let mut file_name = [0u8; PATH_LENGTH];
                let mut start_adjustment = 0;

                if path_len != 0 {
                    for c in value.iter().take(value_len).skip(path_len) {
                        if !(*c == b'\\' || *c == b'/') {
                            break;
                        }
                        start_adjustment += 1;
                    }
                }

                let mut is_filename = true;
                for c in value
                    .iter()
                    .take(value_len)
                    .skip(path_len + start_adjustment)
                {
                    if *c == b'\\' || *c == b'/' {
                        is_filename = false;
                        break;
                    }
                }

                if !is_filename {
                    continue;
                }

                if value_len <= path_len + start_adjustment {
                    continue;
                }

                file_name[..(value_len - path_len - start_adjustment)]
                    .copy_from_slice(&value[path_len + start_adjustment..value_len]);

                result.push(file_name);
            }
        }

        result
    }

    pub fn get_port(&self, name: &[u8]) -> u16 {
        unsafe { (*self.map).get_port(name) }
    }

    pub fn reset(&self, port: u16) {
        self.set(port, &[0; PATH_LENGTH]);
    }
}

#[cfg(test)]
mod tests {
    use iceoryx2_pal_testing::assert_that;

    use crate::platform::win32_udp_port_to_uds_name::PATH_LENGTH;

    use super::PortToUds;

    #[test]
    fn win32_udp_port_to_uds_name_set_and_get_works() {
        let sut = PortToUds::new().unwrap();

        assert_that!(sut.contains(b"hello world"), eq false);
        sut.set(12345, b"hello world");
        assert_that!(sut.contains(b"hello world"), eq true);
        assert_that!(sut.contains(b"some other test"), eq false);
        sut.set(54321, b"some other test");
        assert_that!(sut.contains(b"some other test"), eq true);
        assert_that!(sut.contains(b"fuuu"), eq false);
        sut.set(819, b"fuuu");
        assert_that!(sut.contains(b"fuuu"), eq true);

        assert_that!(sut.get_port(b"hello world"), eq 12345);
        assert_that!(sut.get_port(b"some other test"), eq 54321);
        assert_that!(sut.get_port(b"fuuu"), eq 819);
        assert_that!(sut.get_port(b""), eq 0);
        assert_that!(sut.get_port(b"x"), eq 0);
    }

    #[test]
    fn win32_udp_port_to_uds_name_set_and_get_works_with_multiple_instances() {
        let sut = PortToUds::new().unwrap();

        sut.set(12345, b"hello world");
        sut.set(54321, b"some other test");
        sut.set(819, b"fuuu");
        sut.set(331, b"i am a prime");

        let sut2 = PortToUds::new().unwrap();

        sut2.set(123, b"all glory");
        sut2.set(456, b"to the one and only");
        sut2.set(789, b"hypnotoad");

        assert_that!(sut2.contains(b"hello world"), eq true);
        assert_that!(sut2.contains(b"some other test"), eq true);
        assert_that!(sut2.contains(b"fuuu"), eq true);

        sut2.reset(331);

        assert_that!(sut2.get_port(b"some other test"), eq 54321);
        assert_that!(sut2.get_port(b"fuuu"), eq 819);

        assert_that!(sut.get_port(b"to the one and only"), eq 456);
        assert_that!(sut.get_port(b"hypnotoad"), eq 789);
    }

    fn contains(list: &Vec<[u8; PATH_LENGTH]>, value: &[u8]) -> bool {
        for entry in list {
            if &entry[..value.len()] == value {
                return true;
            }
        }
        false
    }

    #[test]
    fn win32_udp_port_to_uds_list_contents_works() {
        let sut = PortToUds::new().unwrap();

        sut.set(12345, b"/some/uds_1");
        sut.set(12346, b"/some/uds_2.socket");
        sut.set(12347, b"/some/fuu/socket1");
        sut.set(12111, b"/some/fuu/socket2");
        sut.set(12348, b"/some/fuu/socket3");
        sut.set(22348, b"/other///FUN/SOCKET123");
        sut.set(23348, b"//oTher/fun/SockeT456");
        sut.set(12349, b"/socket1/some");
        sut.set(1230, b"/socket4");
        sut.set(1231, b"/socket5");
        sut.set(1232, b"socket6");
        sut.set(1233, b"socket8");
        sut.set(1234, b"socket9");

        let result = sut.list(b"/some/");
        assert_that!(result, len 2);
        assert_that!(contains(&result, b"uds_1"), eq true);
        assert_that!(contains(&result, b"uds_2"), eq true);

        let result = sut.list(b"/");
        assert_that!(result, len 2);
        assert_that!(contains(&result, b"socket4"), eq true);
        assert_that!(contains(&result, b"socket5"), eq true);

        let result = sut.list(b"");
        assert_that!(result, len 3);
        assert_that!(contains(&result, b"socket6"), eq true);
        assert_that!(contains(&result, b"socket8"), eq true);
        assert_that!(contains(&result, b"socket9"), eq true);

        let result = sut.list(b"/some/fuu");
        assert_that!(result, len 3);

        assert_that!(contains(&result, b"socket1"), eq true);
        assert_that!(contains(&result, b"socket2"), eq true);
        assert_that!(contains(&result, b"socket3"), eq true);

        let result = sut.list(b"/other/fun");

        assert_that!(result, len 2);
        assert_that!(contains(&result, b"socket123"), eq true);
        assert_that!(contains(&result, b"socket456"), eq true);
    }
}
