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
    Foundation::{ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND, FALSE},
    Security::{DACL_SECURITY_INFORMATION, SetFileSecurityA},
    Storage::FileSystem::{FILE_ATTRIBUTE_DIRECTORY, GetFileAttributesA, INVALID_FILE_ATTRIBUTES},
};

use crate::posix::{win32_security_attributes::from_mode_to_security_attributes, *};

use crate::win32call;

use super::win32_handle_translator::HandleTranslator;

pub unsafe fn stat(path: *const c_char, buf: *mut stat_t) -> int {
    unsafe {
        if HandleTranslator::get_instance().contains_uds(path.cast()) {
            (*buf).st_mode = S_IFSOCK | S_IRUSR | S_IWUSR | S_IXUSR;
            return 0;
        }
    }

    let (attr, _) = unsafe {
        win32call! { GetFileAttributesA(path as *const u8),
        ignore ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND}
    };
    if attr == INVALID_FILE_ATTRIBUTES {
        Errno::set(unsafe { path_query_errno(path) });
        return -1;
    }

    unsafe {
        if attr & FILE_ATTRIBUTE_DIRECTORY != 0 {
            (*buf).st_mode = S_IFDIR;
        } else {
            (*buf).st_mode = S_IFREG;
        }

        if let Some(mode) = acquire_mode_from_path(core::slice::from_raw_parts(
            path as *const u8,
            c_string_length(path),
        )) {
            (*buf).st_mode |= mode;
        } else {
            Errno::set(path_query_errno(path));
            return -1;
        }
    }
    0
}

pub unsafe fn umask(mask: mode_t) -> mode_t {
    mode_t::MAX
}

unsafe fn path_query_errno(path: *const c_char) -> Errno {
    let original = Errno::get();
    // A concurrent removal can first cause access denial. Only report absence
    // when a subsequent query explicitly confirms it; preserve other failures.
    if matches!(original, Errno::EACCES) {
        let attr = unsafe { GetFileAttributesA(path.cast()) };
        let error = unsafe { windows_sys::Win32::Foundation::GetLastError() };
        if attr == INVALID_FILE_ATTRIBUTES
            && matches!(error, ERROR_FILE_NOT_FOUND | ERROR_PATH_NOT_FOUND)
        {
            return Errno::ENOENT;
        }
    }
    original
}

pub unsafe fn chmod(path: *const c_char, mode: mode_t) -> int {
    let security_attributes = from_mode_to_security_attributes(mode);
    unsafe {
        let (has_file_security_set, _) = win32call!(SetFileSecurityA(
            path.cast(),
            DACL_SECURITY_INFORMATION,
            security_attributes.lpSecurityDescriptor
        ));
        if has_file_security_set == FALSE {
            return -1;
        }

        0
    }
}

// TODO iox2-#2046: Implement Windows support
pub unsafe fn mkfifo(path: *const c_char, mode: mode_t) -> int {
    unimplemented!("mkfifo")
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::{
        Foundation::{ERROR_ACCESS_DENIED, GetLastError},
        Security::{
            Authorization::{
                ConvertStringSecurityDescriptorToSecurityDescriptorA, SDDL_REVISION_1,
            },
            GetFileSecurityA,
        },
        System::Memory::LocalFree,
    };

    #[test]
    fn stat_distinguishes_denied_security_information_from_missing_file() {
        let path = std::env::temp_dir().join(format!(
            "iox2_stat_denied_{}_{}.tmp",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let name = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
        std::fs::write(&path, b"metadata regression").unwrap();
        unsafe {
            let mut metadata = stat_t::new_zeroed();
            assert_eq!(stat(name.as_ptr(), &mut metadata), 0);
            // OWNER RIGHTS suppresses the owner's implicit READ_CONTROL grant. Keep
            // the other owner rights, including DELETE, so the fixture can be removed.
            let mut descriptor = core::ptr::null_mut();
            assert_ne!(
                ConvertStringSecurityDescriptorToSecurityDescriptorA(
                    c"D:P(D;;RC;;;OW)(A;;FA;;;OW)".as_ptr().cast(),
                    SDDL_REVISION_1,
                    &mut descriptor,
                    core::ptr::null_mut(),
                ),
                FALSE
            );
            let set_result =
                SetFileSecurityA(name.as_ptr().cast(), DACL_SECURITY_INFORMATION, descriptor);
            LocalFree(descriptor as isize);
            assert_ne!(set_result, FALSE);
            let attr = GetFileAttributesA(name.as_ptr().cast());
            let mut buffer = [0u8; 1024];
            let mut required = 0;
            let security_result = GetFileSecurityA(
                name.as_ptr().cast(),
                DACL_SECURITY_INFORMATION,
                buffer.as_mut_ptr().cast(),
                buffer.len() as u32,
                &mut required,
            );
            let security_error = GetLastError();
            let denied_result = stat(name.as_ptr(), &mut metadata);
            let denied_errno = Errno::get();
            std::fs::remove_file(&path).unwrap();
            // Model disappearance after a real denial but before classification.
            Errno::set(denied_errno);
            let disappeared_errno = path_query_errno(name.as_ptr());
            Errno::set(Errno::EIO);
            let other_errno = path_query_errno(name.as_ptr());
            let missing_result = stat(name.as_ptr(), &mut metadata);
            let missing_errno = Errno::get();
            assert_ne!(attr, INVALID_FILE_ATTRIBUTES);
            assert_eq!(security_result, FALSE);
            assert_eq!(security_error, ERROR_ACCESS_DENIED);
            assert_eq!(denied_result, -1);
            assert!(
                matches!(denied_errno, Errno::EACCES),
                "denied errno: {denied_errno:?}"
            );
            assert_eq!(missing_result, -1);
            assert!(matches!(missing_errno, Errno::ENOENT));
            assert!(matches!(disappeared_errno, Errno::ENOENT));
            assert!(matches!(other_errno, Errno::EIO));
        }
    }
}
