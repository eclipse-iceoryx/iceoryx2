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

//! Implements [`Serialize`] for the Common Data Representation (cdr),
//! see: <https://en.wikipedia.org/wiki/Common_Data_Representation>.

use crate::serialize::Serialize;
use cdr::{CdrBe, Infinite};
use iceoryx2_bb_log::fail;

use super::{DeserializeError, SerializeError};

/// cdr [`Serialize`]
pub struct Cdr {}

impl Serialize for Cdr {
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SerializeError> {
        Ok(
            fail!(from "Cdr::serialize", when cdr::serialize::<_, _, CdrBe>(&value, Infinite),
                with SerializeError::InternalError, "Failed to serialize object" ),
        )
    }

    fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, DeserializeError> {
        Ok(
            fail!(from "Cdr::deserialize", when cdr::deserialize::<T>(bytes),
                    with DeserializeError::InternalError, "Failed to deserialize object."),
        )
    }
}
