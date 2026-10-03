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

#![allow(dead_code)]
#![allow(unused_variables)]

use windows_sys::Win32::Foundation::{ERROR_SUCCESS, FALSE, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Security::Authorization::{
    ConvertSecurityDescriptorToStringSecurityDescriptorA,
    ConvertStringSecurityDescriptorToSecurityDescriptorA,
};
use windows_sys::Win32::Security::Authorization::{
    ConvertSidToStringSidA, GetSecurityInfo, SDDL_REVISION_1, SE_FILE_OBJECT,
};
use windows_sys::Win32::Security::{
    ACL, DACL_SECURITY_INFORMATION, GetSecurityDescriptorOwner, OWNER_SECURITY_INFORMATION,
    PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
};
use windows_sys::Win32::System::Memory::LocalFree;

use crate::posix::{S_IRGRP, S_IROTH, S_IWGRP, S_IWOTH, S_IXGRP, S_IXOTH, mode_t};
use crate::posix::{S_IRUSR, S_IWUSR, S_IXUSR, types::*};

// SID String Mappings
// syntax:
// ace_type;ace_flags;rights;object_guid;inherit_object_guid;account_sid
// ace_type:
//     A = allow
//     D = deny
// ace_flags:
//     OI = OBJECT_INHERIT_ACE
//     CI = CONTAINER_INHERIT_ACE
// rights:
//     GA = Generic All
//     GR = Generic Read
//     GW = Generic Write
//     GX = Generic Execute
//     SD = Delete permission
// object_guid:
//     empty
// inherit_object_guid:
//     empty
// account_sid:
//     WD - Everyone (posix: others)
//     SY - Local System
//     SU - Service Logon User (group) (posix: group)
//     PS - Principal Self
//     OW - Owner Rights (posix: owner)
//     CO - Creator Owner
//     CG - Creator Group
//     BA - Builtin Administrator
//     BG - Builtin Guests
//     BU - Builtin Users

// https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings
// https://learn.microsoft.com/en-us/windows/win32/secauthz/sid-strings
const SID_LENGTH: usize = 255;
const GENERIC_PERM_ALL: &[u8] = b"GA";
const GENERIC_PERM_READ: &[u8] = b"GR";
const GENERIC_PERM_WRITE: &[u8] = b"GW";
const GENERIC_PERM_EXECUTE: &[u8] = b"GX";
const FILE_PERM_ALL: &[u8] = b"FA";
const FILE_PERM_READ: &[u8] = b"FR";
const FILE_PERM_WRITE: &[u8] = b"FW";
const FILE_PERM_EXECUTE: &[u8] = b"FX";
const ACE_INHERITANCE: &[u8] = b"OICI";
const DELETE: &[u8] = b"SD";

const IDENT_OTHERS: &[u8] = b"WD";
const IDENT_GROUP: &[u8] = b"SU";
const IDENT_OWNER: &[u8] = b"BU";

// Windows access rights constants as HEX
// https://learn.microsoft.com/en-us/windows/win32/fileio/file-access-rights-constants
// https://learn.microsoft.com/en-us/windows/win32/secauthz/access-mask
const FILE_READ_DATA_HEX: u32 = 0x1;
const FILE_WRITE_DATA_HEX: u32 = 0x2;
const FILE_EXECUTE_HEX: u32 = 0x20;
const FILE_READ_ATTRIBUTES_HEX: u32 = 0x80;
const FILE_WRITE_ATTRIBUTES_HEX: u32 = 0x100;
const DELETE_HEX: u32 = 0x10000;

fn add_to_sd_string(data: &mut [u8], add: &[u8]) {
    let mut start_adding = false;
    let mut start = 0;

    for i in 0..data.len() {
        if data[i] == b'\0' && !start_adding {
            start_adding = true;
            start = i;
        }

        if start_adding {
            if i - start >= add.len() {
                data[i] = b'\0';
                break;
            } else {
                data[i] = add[i - start];
            }
        }
    }
}

macro_rules! add_to_ace_string {
    ($data:expr, $($value:expr),*) => {
        $(add_to_sd_string($data, ($value)));*
    };
}

fn get_owner_sid(handle: HANDLE) -> [u8; SID_LENGTH] {
    let mut sid = [0u8; SID_LENGTH];
    if handle == INVALID_HANDLE_VALUE {
        sid[0] = IDENT_OWNER[0];
        sid[1] = IDENT_OWNER[1];
        return sid;
    }

    let mut security_descriptor: PSECURITY_DESCRIPTOR = core::ptr::null_mut::<void>();

    if unsafe {
        GetSecurityInfo(
            handle,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION,
            core::ptr::null_mut::<*mut void>(),
            core::ptr::null_mut::<*mut void>(),
            core::ptr::null_mut::<*mut ACL>(),
            core::ptr::null_mut::<*mut ACL>(),
            &mut security_descriptor,
        )
    } != ERROR_SUCCESS
    {
        return sid;
    }

    let mut owner_sid = core::ptr::null_mut::<void>();
    let mut owner_defaulted = FALSE;

    let (has_read_owner, _) = unsafe {
        win32call! {GetSecurityDescriptorOwner(security_descriptor, &mut owner_sid, &mut owner_defaulted)}
    };
    if has_read_owner == FALSE {
        unsafe {
            win32call! {LocalFree(security_descriptor as isize) }
        };
        return sid;
    }

    let mut owner_str = core::ptr::null_mut::<u8>();

    let (convert_result, _) = unsafe {
        win32call! {ConvertSidToStringSidA(owner_sid, &mut owner_str)}
    };
    if convert_result == FALSE {
        unsafe {
            win32call! {LocalFree(security_descriptor as isize) }
        };
        return sid;
    }

    unsafe {
        win32call! {LocalFree(security_descriptor as isize)}
    };
    unsafe {
        win32call! {LocalFree(owner_str as isize)}
    };

    for (i, sid_element) in sid.iter_mut().enumerate().take(SID_LENGTH) {
        let c = unsafe { *owner_str.add(i) };
        if c == b'\0' {
            break;
        }
        *sid_element = c;
    }

    sid
}

pub fn from_mode_to_security_attributes(mode: mode_t) -> SECURITY_ATTRIBUTES {
    let mut attr = SECURITY_ATTRIBUTES {
        nLength: core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: core::ptr::null_mut::<void>(),
        bInheritHandle: FALSE,
    };

    let mut buffer = [0u8; 1024];

    // deny everything
    add_to_ace_string!(&mut buffer, b"D:");

    // local system allow everything
    add_to_ace_string!(
        &mut buffer,
        b"(A;",
        ACE_INHERITANCE,
        b";",
        GENERIC_PERM_ALL,
        b";;;SY)"
    );

    // builtin administrator allow everything
    add_to_ace_string!(
        &mut buffer,
        b"(A;",
        ACE_INHERITANCE,
        b";",
        GENERIC_PERM_ALL,
        b";;;BA)"
    );

    let add_permissions = |buffer: &mut [u8], ident: &[u8]| {
        // determine which permissions to set based on identity
        let (read_bit, write_bit, exec_bit) = match ident {
            IDENT_OTHERS => (S_IROTH, S_IWOTH, S_IXOTH),
            IDENT_GROUP => (S_IRGRP, S_IWGRP, S_IXGRP),
            IDENT_OWNER => (S_IRUSR, S_IWUSR, S_IXUSR),
            _ => panic!(
                "Attempted to set permissions for unknown identity: {:?}",
                ident
            ),
        };

        let has_read = mode & read_bit != 0;
        let has_write = mode & write_bit != 0;
        let has_exec = mode & exec_bit != 0;

        if has_read && has_write && has_exec {
            // all permissions, including delete
            add_to_ace_string!(
                buffer,
                b"(A;",
                ACE_INHERITANCE,
                b";",
                GENERIC_PERM_ALL,
                DELETE,
                b";;;",
                ident,
                b")"
            );
        } else {
            // individual permissions - use separate ACEs for each
            if has_read {
                add_to_ace_string!(
                    buffer,
                    b"(A;",
                    ACE_INHERITANCE,
                    b";",
                    GENERIC_PERM_READ,
                    b";;;",
                    ident,
                    b")"
                );
            }
            if has_write {
                // include delete permissions with write permissions
                add_to_ace_string!(
                    buffer,
                    b"(A;",
                    ACE_INHERITANCE,
                    b";",
                    GENERIC_PERM_WRITE,
                    DELETE,
                    b";;;",
                    ident,
                    b")"
                );
            }
            if has_exec {
                add_to_ace_string!(
                    buffer,
                    b"(A;",
                    ACE_INHERITANCE,
                    b";",
                    GENERIC_PERM_EXECUTE,
                    b";;;",
                    ident,
                    b")"
                );
            }
        }
    };

    // add permissions for each category
    add_permissions(&mut buffer, IDENT_OTHERS);
    add_permissions(&mut buffer, IDENT_GROUP);
    add_permissions(&mut buffer, IDENT_OWNER);

    // A read-only object must stay removable by its owner. On POSIX, deleting a
    // file is a property of its parent directory (write plus execute), never of
    // the file mode, so a 0400 file is removable by the user who owns it.
    // Windows instead requires DELETE on the object itself, a right the mode
    // cannot express, so it is granted here independently of the write bit.
    //
    // Without it, a read-only file is indestructible for its own creator. The
    // state of every process that was killed - `shm_unlink` never ran, so the
    // files stay - becomes permanent, and the dead node cleanup that is meant
    // to reap it (`Node::blocking_cleanup_dead_nodes`, which deletes as the
    // calling user) fails with ERROR_ACCESS_DENIED on that same user and skips
    // the node instead of failing, leaving the state on the machine forever.
    //
    // The owner-rights SID (`OW`, S-1-3-4) cannot be used for this: Windows
    // silently *drops* an OWNER RIGHTS ACE when a DACL carrying one is written
    // to a file, so it never reaches the object. The identity that already
    // holds the POSIX owner bits is the one that must carry the right.
    add_to_ace_string!(
        &mut buffer,
        b"(A;",
        ACE_INHERITANCE,
        b";",
        DELETE,
        b";;;",
        IDENT_OWNER,
        b")"
    );

    let (convert_result, _) = unsafe {
        win32call! { ConvertStringSecurityDescriptorToSecurityDescriptorA(
            buffer.as_ptr(),
            SDDL_REVISION_1,
            &mut attr.lpSecurityDescriptor,
            core::ptr::null_mut::<u32>(),
        ) }
    };

    if convert_result == FALSE {
        return SECURITY_ATTRIBUTES {
            nLength: 0,
            lpSecurityDescriptor: core::ptr::null_mut::<void>(),
            bInheritHandle: FALSE,
        };
    }

    attr
}

fn extract_ace_entry_sections(entry: &[u8]) -> (&[u8], &[u8], &[u8]) {
    let mut iter = entry.split(|c| *c == b';');

    let ace_type = iter.next().unwrap();

    // ace flags
    iter.next();

    let ace_rights = iter.next().unwrap();

    // object guid
    iter.next();
    // inherit object guid
    iter.next();

    let account_sid = iter.next().unwrap();

    (ace_type, ace_rights, account_sid)
}

fn extract_ace_entries(value: &[u8]) -> Vec<&[u8]> {
    let mut ace_entries: Vec<&[u8]> = vec![];

    let mut bracket_counter = 0;
    let mut entry_start = 0;
    for i in 0..value.len() {
        if value[i] == b'(' {
            bracket_counter += 1;
            if bracket_counter == 1 {
                entry_start = i + 1;
            }
        }
        if value[i] == b')' && bracket_counter > 0 {
            bracket_counter -= 1;
            if bracket_counter == 0 {
                ace_entries.push(&value[entry_start..i])
            }
        }
    }

    ace_entries
}

fn parse_hex_rights(hex_str: &str) -> u8 {
    let mut ret_val = 0;
    if let Ok(hex_val) = u32::from_str_radix(hex_str, 16) {
        if (hex_val & FILE_READ_DATA_HEX) != 0 || (hex_val & FILE_READ_ATTRIBUTES_HEX) != 0 {
            ret_val |= 4;
        }
        // `DELETE` is granted on its own, because the POSIX mode cannot express
        // it: only the write bit maps to the POSIX write mode.
        if (hex_val & FILE_WRITE_DATA_HEX) != 0 || (hex_val & FILE_WRITE_ATTRIBUTES_HEX) != 0 {
            ret_val |= 2;
        }
        if (hex_val & FILE_EXECUTE_HEX) != 0 {
            ret_val |= 1;
        }
    }
    ret_val
}

fn parse_ace_string_rights(ace_rights: &[u8]) -> u8 {
    let mut i = 0;
    let mut ret_val = 0;

    while i < ace_rights.len() {
        if i + 2 > ace_rights.len() {
            break;
        }

        let right = &ace_rights[i..i + 2];

        if right == GENERIC_PERM_ALL || right == FILE_PERM_ALL {
            ret_val = 7;
        }
        if right == GENERIC_PERM_READ || right == FILE_PERM_READ {
            ret_val |= 4;
        }
        // `DELETE` is granted on its own, because the POSIX mode cannot express
        // it: only the write bit maps to the POSIX write mode.
        if right == GENERIC_PERM_WRITE || right == FILE_PERM_WRITE {
            ret_val |= 2;
        }
        if right == GENERIC_PERM_EXECUTE || right == FILE_PERM_EXECUTE {
            ret_val |= 1;
        }

        i += 2;
    }

    ret_val
}

fn ace_rights_to_bits(ace_rights: &[u8]) -> u8 {
    if ace_rights.starts_with(b"0x")
        && let Ok(hex_str) = core::str::from_utf8(&ace_rights[2..])
    {
        return parse_hex_rights(hex_str);
    }
    parse_ace_string_rights(ace_rights)
}

fn is_owner(account_sid: &[u8]) -> bool {
    account_sid == IDENT_OWNER || (account_sid.len() >= 2 && &account_sid[0..2] == b"S-")
}

pub fn from_security_attributes_to_mode(value: &SECURITY_ATTRIBUTES) -> mode_t {
    let mut mode: mode_t = 0;
    let mut raw_string: *mut u8 = core::ptr::null_mut();
    let mut raw_string_length = 0;
    let (convert_result, _) = unsafe {
        win32call! { ConvertSecurityDescriptorToStringSecurityDescriptorA(
                value.lpSecurityDescriptor,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut raw_string,
                &mut raw_string_length,
            )
        }
    };
    if convert_result == FALSE {
        return mode;
    }

    let raw_string = unsafe { core::slice::from_raw_parts(raw_string, raw_string_length as usize) };
    let ace_entries = extract_ace_entries(raw_string);

    // group ACEs by identity
    let mut others_perms = 0u8;
    let mut group_perms = 0u8;
    let mut owner_perms = 0u8;

    for entry in ace_entries {
        let (ace_type, ace_rights, account_sid) = extract_ace_entry_sections(entry);

        if ace_type == b"D" {
            continue;
        }

        let rights_bits = ace_rights_to_bits(ace_rights);

        if account_sid == IDENT_OTHERS {
            others_perms |= rights_bits;
        } else if account_sid == IDENT_GROUP {
            group_perms |= rights_bits;
        } else if is_owner(account_sid) {
            owner_perms |= rights_bits;
        }
    }

    // consolidate permissions into single POSIX mode
    mode |= others_perms as u64;
    mode |= (group_perms as u64) << 3;
    mode |= (owner_perms as u64) << 6;

    mode
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders the DACL of `attr` as its SDDL string and hands it to `check`.
    fn with_dacl_string(attr: &SECURITY_ATTRIBUTES, check: impl FnOnce(&str)) {
        let mut raw: *mut u8 = core::ptr::null_mut();
        let mut raw_length = 0;
        let (converted, _) = unsafe {
            win32call! { ConvertSecurityDescriptorToStringSecurityDescriptorA(
                attr.lpSecurityDescriptor,
                SDDL_REVISION_1,
                DACL_SECURITY_INFORMATION,
                &mut raw,
                &mut raw_length,
            ) }
        };
        assert_ne!(converted, FALSE, "the descriptor must render as SDDL");

        let bytes = unsafe { core::slice::from_raw_parts(raw, raw_length as usize) };
        check(core::str::from_utf8(bytes).expect("SDDL is an ASCII string"));
    }

    /// A read-only mode must not make the object undeletable for its owner.
    ///
    /// The POSIX mode cannot express deletion: on POSIX the parent directory
    /// governs it, so a 0400 file stays removable by its owner. Windows needs
    /// the right on the object itself, which is why it is granted here whatever
    /// the mode. Without it the state every killed process leaves behind
    /// survives forever: the user cannot remove it, and
    /// [`Node::blocking_cleanup_dead_nodes`] - which deletes as that same user -
    /// receives ERROR_ACCESS_DENIED and skips the node instead of failing.
    #[test]
    fn a_read_only_mode_still_grants_delete_to_its_owner() {
        with_dacl_string(&from_mode_to_security_attributes(S_IRUSR), |dacl| {
            assert!(
                dacl.contains("SD;;;BU)"),
                "the POSIX owner identity must be able to delete: {dacl}"
            );
        });
    }

    /// The reverse mapping is what `stat` reports as the file mode, so `DELETE`
    /// must no longer be read as a proxy for the POSIX write bit.
    #[test]
    fn the_mode_round_trip_survives_the_owner_delete_ace() {
        for mode in [
            S_IRUSR,
            S_IWUSR,
            S_IXUSR,
            S_IRUSR | S_IWUSR,
            S_IRUSR | S_IXUSR,
            S_IRGRP | S_IROTH,
            S_IRUSR | S_IWUSR | S_IXUSR,
        ] {
            let attr = from_mode_to_security_attributes(mode);
            assert_eq!(
                from_security_attributes_to_mode(&attr),
                mode,
                "mode {mode:o} must survive the round trip"
            );
        }
    }
}
