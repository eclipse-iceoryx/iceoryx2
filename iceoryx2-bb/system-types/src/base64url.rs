// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

pub use iceoryx2_bb_container::semantic_string::SemanticString;

use core::hash::{Hash, Hasher};
use iceoryx2_bb_container::semantic_string;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_pal_configuration::FILENAME_LENGTH;

use crate::file_name::FileName;

semantic_string! {
  /// Represents a Base64 encoding according to: RFC 4648 §5 (URL- and filename-safe standard)
  /// The optional padding character `=` is not allowed.
  name: Base64Url,
  capacity: FILENAME_LENGTH,
  invalid_content: |value: &[u8]| {
    value.is_empty()
  },
  invalid_characters: |value: &[u8]| {
    value.iter().any(|c| {
        !(c.is_ascii_lowercase() ||
          c.is_ascii_uppercase() ||
          c.is_ascii_digit() ||
          *c == b'-' || *c == b'_')
    })
  },
  normalize: |this: &Base64Url| {
      this.clone()
  }
}

impl Base64Url {
    pub fn as_file_name(&self) -> FileName {
        // SAFETY: Base64Url contains always a valid file name
        unsafe { FileName::new_unchecked(self.as_bytes()) }
    }
}
