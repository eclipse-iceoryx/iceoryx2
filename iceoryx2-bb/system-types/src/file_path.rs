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
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_bb_system_types::file_path::*;
//!
//! let name = FilePath::new(b"some/path/../bla/some_file.txt");
//!
//! let invalid_name = FilePath::new(b"final/slash/indicates/directory/");
//! assert!(invalid_name.is_err());
//! ```

pub use iceoryx2_bb_container::semantic_string::SemanticString;

use crate::file_name::FileName;
use crate::path::Path;
use core::hash::{Hash, Hasher};
use iceoryx2_bb_container::byte_string::FixedSizeByteString;
use iceoryx2_bb_container::semantic_string;
use iceoryx2_bb_container::semantic_string::SemanticStringError;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_configuration::{PATH_LENGTH, PATH_SEPARATOR};

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
    for c in value {
        match c {
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
      this.clone()
  }
}

impl FilePath {
    /// Creates a new [`FilePath`] from a given [`Path`] and [`FileName`]
    pub fn from_path_and_file(path: &Path, file: &FileName) -> Result<Self, SemanticStringError> {
        let msg = "Unable to create FilePath from path and file";
        let mut new_self = Self {
            value: unsafe { FixedSizeByteString::new_unchecked(path.as_bytes()) },
        };

        if !path.is_empty() && path.as_bytes()[path.len() - 1] != PATH_SEPARATOR {
            fail!(from "FilePath::from_path_and_file()", when new_self.value.push(PATH_SEPARATOR),
                with SemanticStringError::ExceedsMaximumLength,
                "{} since the concatination would exceed the maximum supported length of {}.",
                msg, PATH_LENGTH);
        }

        fail!(from "FilePath::from_path_and_file()", when new_self.value.push_bytes(file.as_bytes()),
                with SemanticStringError::ExceedsMaximumLength,
                "{} since the concatination would exceed the maximum supported length of {}.",
                msg, PATH_LENGTH);

        Ok(new_self)
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
