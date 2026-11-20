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

use iceoryx2_bb_flatbuffers::reflection::{self, BaseType, Object, Schema};
use iceoryx2_log::{fail, origin};

use super::{PairingError, Refusal};

/// The field of `object` named `name`.
pub(super) fn field<'a>(
    object: Object<'a>,
    name: &str,
) -> Result<reflection::Field<'a>, PairingError> {
    let origin = origin!("pairing::flatbuffer::field");

    let Some(field) = object.fields().iter().find(|field| field.name() == name) else {
        fail!(
            from origin,
            with PairingError { path: String::new(), refusal: Refusal::MissingField },
            "Table '{}' has no field '{}'", object.name(), name
        );
    };
    Ok(field)
}

/// The object a field of type `Obj` refers to.
pub(super) fn object<'a>(schema: &Schema<'a>, index: i32) -> Object<'a> {
    schema
        .objects()
        .get(usize::try_from(index).expect("an Obj field refers to an object"))
}

/// The type of `field` as written in a schema, `[double]`.
pub(super) fn field_type(field: reflection::Field<'_>) -> String {
    let ty = field.type_();
    match ty.base_type() {
        BaseType::Vector => format!("[{}]", ty.element().variant_name().unwrap_or("?")),
        base_type => base_type.variant_name().unwrap_or("?").to_string(),
    }
}
