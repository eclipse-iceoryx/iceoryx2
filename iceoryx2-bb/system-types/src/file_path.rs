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

//! Relocatable (inter-process shared memory compatible) [`SemanticString`] implementation for
//! [`FilePath`]. All modification operations ensure that never an
//! invalid file or path name can be generated. All strings have a fixed size so that the maximum
//! path or file name length the system supports can be stored.
//!
//! # Example
//!
//! ```ignore
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_bb_system_types::file_path::*;
//!
//! let name = FilePath::new(b"some/path/../bla/some_file.txt");
//!
//! let invalid_name = FilePath::new(b"final/slash/indicates/directory/");
//! assert!(invalid_name.is_err());
//! ```

pub use iceoryx2_bb_container::semantic_string::SemanticString;

use core::hash::{Hash, Hasher};

use alloc::string::String;

use iceoryx2_bb_container::semantic_string;
use iceoryx2_bb_container::semantic_string::SemanticStringError;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;
use iceoryx2_pal_configuration::{PATH_LENGTH, PATH_SEPARATOR};

use crate::file_name::FileName;
use crate::path::Path;

semantic_string! {
  /// Represents a file path. The restriction are choosen in a way that it is platform independent.
  /// This means characters/strings which would be legal on some platforms are forbidden as well.
  name: FilePath,
  capacity: PATH_LENGTH,
  invalid_content: |string: &[u8]| {
    match string {
        b"" => return true,
        b"." => return true,
        b".." => return true,
        _ => (),
    }

    // only directories can end with /
    if string[string.len() - 1] == PATH_SEPARATOR {
        return true;
    }

    // everything which ends with /. is invalid
    if string.len() >= 2 && string.get(string.len() - 2..string.len()).unwrap() == [PATH_SEPARATOR, b'.'] {
        return true;
    }

    // everything which ends with /.. is invalid
    if string.len() >= 3 && string.get(string.len() - 3..string.len()).unwrap() == [PATH_SEPARATOR, b'.', b'.'] {
        return true;
    }

    false
  },
  invalid_characters: |value: &[u8]| {
    for (index, character) in value.iter().enumerate() {
        if index != 1 { // paths like C:\fuu must be allowed
            #[cfg(target_os = "windows")]
            if *character == b':' {
                return true
            }
        }

        match character {
            // linux & windows
            0 => return true,
            // windows only
            1..=31 => return true,
            b'<' => return true,
            b'>' => return true,
            b'"' => return true,
            b'|' => return true,
            b'?' => return true,
            b'*' => return true,
            _ => (),
        }
    }

    false
  },
  normalize: |this: &FilePath| {
      *this
  }
}

impl FilePath {
    /// Creates a new [`FilePath`] from a given [`Path`] and [`FileName`]
    pub fn from_path_and_file(path: &Path, file: &FileName) -> Result<Self, SemanticStringError> {
        let msg = "Unable to create FilePath from path and file";
        let mut required_len = path.len() + file.len();
        if !path.is_empty() && path.as_bytes()[path.len() - 1] != PATH_SEPARATOR {
            required_len += 1;
        }

        if PATH_LENGTH < required_len {
            fail!(from "FilePath::from_path_and_file()",
                with SemanticStringError::ExceedsMaximumLength,
                "{} since the concatination of \"{}\" and \"{}\" would exceed the maximum supported length of {}.",
                msg, path, file, PATH_LENGTH);
        }

        Ok(unsafe { Self::from_path_and_file_unchecked(path, file) })
    }

    /// Creates a new [`FilePath`] from a given [`Path`] and [`FileName`]
    ///
    /// # Safety
    ///
    /// * [`Path::len()`] + [`FileName::len()`] + 1 <= [`FilePath::max_len()`]
    ///
    pub const unsafe fn from_path_and_file_unchecked(path: &Path, file: &FileName) -> Self {
        debug_assert!(path.as_bytes_const().len() + file.as_bytes_const().len() + 1 < PATH_LENGTH);

        let mut buffer = [0u8; PATH_LENGTH];
        let mut buffer_len = path.as_bytes_const().len();

        core::ptr::copy_nonoverlapping(
            path.as_bytes_const().as_ptr(),
            buffer.as_mut_ptr(),
            buffer_len,
        );

        if 0 < buffer_len && path.as_bytes_const()[buffer_len - 1] != PATH_SEPARATOR {
            core::ptr::copy_nonoverlapping(&PATH_SEPARATOR, buffer.as_mut_ptr().add(buffer_len), 1);
            buffer_len += 1;
        }

        let file_len = file.as_bytes_const().len();
        core::ptr::copy_nonoverlapping(
            file.as_bytes_const().as_ptr(),
            buffer.as_mut_ptr().add(buffer_len),
            file_len,
        );
        buffer_len += file_len;

        Self {
            value: iceoryx2_bb_container::string::StaticString::from_bytes_unchecked_restricted(
                &buffer, buffer_len,
            ),
        }
    }

    /// Returns the last file part ([`FileName`]) of the path.
    pub fn file_name(&self) -> FileName {
        let file_name = self
            .as_bytes()
            .rsplitn(2, |c| *c == PATH_SEPARATOR)
            .next()
            .unwrap();
        // SAFETY
        // * the file path ensures that is a valid path to a file, therefore the last part
        //   must be a valid FileName
        unsafe { FileName::new_unchecked(file_name) }
    }

    /// Returns the [`Path`] part of the [`FilePath`].
    pub fn path(&self) -> Path {
        let path = match self.as_bytes().rsplitn(2, |c| *c == PATH_SEPARATOR).nth(1) {
            Some(p) if !p.is_empty() => p,
            Some(_) => &[PATH_SEPARATOR],
            None => &[],
        };
        // SAFETY
        // * the file path ensures that is a valid path to a file, therefore the first part
        //   must be a valid path
        unsafe { Path::new_unchecked(path) }
    }
}

impl From<FileName> for FilePath {
    fn from(value: FileName) -> Self {
        unsafe { FilePath::new_unchecked(value.as_bytes()) }
    }
}
