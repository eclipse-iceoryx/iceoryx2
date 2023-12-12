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

//! Relocatable (inter-process shared memory compatible) [`semantic_string::SemanticString`] implementation for
//! [`FileName`]. All modification operations ensure that never an
//! invalid file or path name can be generated. All strings have a fixed size so that the maximum
//! path or file name length the system supports can be stored.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_bb_system_types::file_name::*;
//!
//! let name = FileName::new(b"some_file.txt");
//!
//! let invalid_name = FileName::new(b"no/path/allowed.txt");
//! assert!(invalid_name.is_err());
//! ```

use iceoryx2_bb_container::semantic_string;
use iceoryx2_pal_settings::FILENAME_LENGTH;

semantic_string! {
  /// Represents a file name. The restriction are choosen in a way that it is platform independent.
  /// This means characters/strings which would be legal on some platforms are forbidden as well.
  name: FileName,
  capacity: FILENAME_LENGTH,
  invalid_content: |value: &[u8]| {
    matches!(value, b"" | b"." | b"..")
  },
  invalid_characters: |value: &[u8]| {
    for c in value {
        match c {
            // linux & windows
            0 => return true,
            b'/' => return true,
            // windows only
            1..=31 => return true,
            b':' => return true,
            b'\\' => return true,
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
  comparision: |lhs: &[u8], rhs: &[u8]| {
      *lhs == *rhs
  }
}
