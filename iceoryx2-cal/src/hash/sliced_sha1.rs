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

//! Creates a Sha1 [`Hash`]. **Shall not be used for security critical use cases.**

use crate::hash::*;
use sha1_smol::Digest;

pub struct Sha1 {
    hash: Digest,
}

impl ToB64 for Sha1 {
    fn to_b64(&self) -> String {
        let mut adjusted_bytes = [0u8; 16];
        adjusted_bytes.copy_from_slice(&self.hash.bytes()[0..16]);
        adjusted_bytes.to_b64()
        //        self.hash.bytes().to_b64()
    }
}

impl Hash for Sha1 {
    fn new(bytes: &[u8]) -> Self {
        Self {
            hash: {
                let mut hash = sha1_smol::Sha1::new();
                hash.update(bytes);
                hash.digest()
            },
        }
    }
}
