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

use crate::serialize::Serialize;
use iceoryx2_bb_log::fail;

use super::{DeserializeError, SerializeError};

pub struct Postcard {}

impl Serialize for Postcard {
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, super::SerializeError> {
        match postcard::to_allocvec(value) {
            Ok(vec) => Ok(vec),
            Err(e) => {
                fail!(
                    from "Postcard::serialize",
                    with SerializeError::InternalError,
                    "Failed to serialize object: {e}"
                );
            }
        }
    }

    fn deserialize<T: serde::de::DeserializeOwned>(
        bytes: &[u8],
    ) -> Result<T, super::DeserializeError> {
        match postcard::from_bytes(bytes) {
            Ok(obj) => Ok(obj),
            Err(e) => {
                fail!(
                    from "Postcard::deserialize",
                    with DeserializeError::InternalError,
                    "Failed to deserialize object: {e}"
                );
            }
        }
    }
}
