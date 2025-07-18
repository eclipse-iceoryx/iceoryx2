// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use alloc::vec::Vec;

use crate::serialize::Serialize;
use iceoryx2_bb_log::fail;

use super::{DeserializeError, SerializeError};

pub struct Bincode {}

impl Serialize for Bincode {
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, super::SerializeError> {
        let config = bincode::config::standard();
        match bincode::serde::encode_to_vec(value, config) {
            Ok(result) => Ok(result),
            Err(e) => {
                fail!(
                    from "Bincode::serialize",
                    with SerializeError::InternalError,
                    "Failed to serialize object: ({e})"
                );
            }
        }
    }

    fn deserialize<T: serde::de::DeserializeOwned>(
        bytes: &[u8],
    ) -> Result<T, super::DeserializeError> {
        let config = bincode::config::standard();
        match bincode::serde::decode_from_slice(bytes, config) {
            Ok((result, _len)) => Ok(result),
            Err(e) => {
                fail!(
                    from "Bincode::deserialize",
                    with DeserializeError::InternalError,
                    "Failed to deserialize object: ({e})"
                );
            }
        }
    }
}
