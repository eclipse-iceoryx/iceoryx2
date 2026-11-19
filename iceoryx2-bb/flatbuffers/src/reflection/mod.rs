// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

pub use iceoryx2_bb_flatbuffers_generated::binding::reflection::*;

use iceoryx2_log::fail;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaError {
    NotABinarySchema,
    NoRootType,
}

impl core::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SchemaError::{self:?}")
    }
}

impl core::error::Error for SchemaError {}

/// The fully qualified name of the root table `schema` declares.
///
/// Fails when `schema` is not a binary flatbuffer schema or declares no
/// `root_type`.
pub fn root_type_name(schema: &[u8]) -> Result<&str, SchemaError> {
    let origin = "root_type_name()";

    let schema = fail!(
        from origin,
        when root_as_schema(schema),
        with SchemaError::NotABinarySchema,
        "The {} bytes are not a binary flatbuffer schema.", schema.len()
    );
    let Some(root_table) = schema.root_table() else {
        fail!(
            from origin,
            with SchemaError::NoRootType,
            "The schema declares no root_type. Add `root_type <Table>;` to the .fbs and regenerate the binary schema."
        );
    };
    Ok(root_table.name())
}
