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

mod introspection;
mod schema;

use iceoryx2_bb_flatbuffers::reflection::{self, BaseType, Object, root_as_schema};
use iceoryx2_integrations_ros2_link_adapter::typesupport::{IntrospectionKind, TypeSupport};
use iceoryx2_link_backend::service_description::Schema;
use iceoryx2_log::{fail, origin};
use r2r_rcl::{
    rosidl_typesupport_introspection_c__MessageMember as MessageMember,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
};

use super::flatbuffer::{FieldPlacement, Slot, TableIndex};
use super::model::{Kind, Model, ValueIndex};
use super::{cdr, flatbuffer};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Compiled {
    pub model: Model,
    pub cdr: cdr::Layout,
    pub flatbuffer: flatbuffer::Layout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    /// The path of the member from the message e.g. `header.stamp.sec`.
    pub path: String,
    pub refusal: Refusal,
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {:?}", self.path, self.refusal)
    }
}

impl core::error::Error for CompileError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The member has no field of its name.
    MissingField,
    /// The member and the field carry different kinds of value.
    Mismatch { member: String, field: String },
    /// No flatbuffer field carries the type of the member.
    Unsupported { member: String },
}

/// Compiles the model of the message that `schema` and `typesupport` both
/// describe, with its layout in each of the two formats.
///
/// Fails when the two do not describe the same message.
pub fn compile(
    schema: &Schema,
    typesupport: &TypeSupport<IntrospectionKind>,
) -> Result<Compiled, CompileError> {
    let origin = origin!("compile");

    // Find the root table of the schema.
    let schema = root_as_schema(schema.bytes()).expect("a schema holds a binary flatbuffer schema");
    let root = schema.root_table().expect("a schema declares a root type");

    // Walk the message and the root table together.
    let mut compiler = Compiler {
        compiled: Compiled::default(),
    };

    // SAFETY: the typesupport keeps the members valid while it is loaded.
    if let Err(error) = unsafe { compiler.add_table(root, typesupport.members(), "") } {
        fail!(
            from origin,
            with error,
            "Member '{}' has no value in '{}'", error.path, root.name()
        );
    }

    Ok(compiler.compiled)
}

struct Compiler {
    compiled: Compiled,
}

impl Compiler {
    /// Adds the table that holds a message.
    ///
    /// # Safety
    ///
    /// `message` must point to the members of a loaded introspection typesupport.
    unsafe fn add_table(
        &mut self,
        object: Object<'_>,
        message: *const MessageMembers,
        path: &str,
    ) -> Result<(), CompileError> {
        // Add the table to the flatbuffer layout.
        let table = TableIndex(self.compiled.flatbuffer.tables.len() as u16);
        self.compiled
            .flatbuffer
            .tables
            .push(flatbuffer::TableEntry { values: Vec::new() });

        for member in unsafe { introspection::members(message) } {
            let name = unsafe { introspection::member_name(member) };
            let path = member_path(path, name);
            let field = schema::field(object, name).map_err(|refusal| CompileError {
                path: path.clone(),
                refusal,
            })?;

            // Check that the member and the field hold the same kind of value.
            let (kind, member_kind, field_kind) =
                kinds(member, field).map_err(|refusal| CompileError { path, refusal })?;

            let placement = FieldPlacement::Table(Slot {
                table,
                vtable_offset: field.offset(),
            });
            let index = self.add_value(kind, member_kind, placement, field_kind);
            self.compiled.flatbuffer.tables[usize::from(table.0)]
                .values
                .push(index);
        }

        Ok(())
    }

    /// Adds a value of `kind` with its member and its field, returning its
    /// index.
    fn add_value(
        &mut self,
        kind: Kind,
        member_kind: cdr::MemberKind,
        placement: FieldPlacement,
        field_kind: flatbuffer::FieldKind,
    ) -> ValueIndex {
        // Add the value to the model.
        let index = ValueIndex(self.compiled.model.values.len() as u16);
        self.compiled.model.values.push(kind);

        // Add the member of the value to the CDR layout.
        self.compiled
            .cdr
            .members
            .push(cdr::MemberEntry { kind: member_kind });
        self.compiled.cdr.order.push(index);

        // Add the field of the value to the flatbuffer layout.
        self.compiled
            .flatbuffer
            .fields
            .push(flatbuffer::FieldEntry {
                placement,
                kind: field_kind,
            });

        index
    }
}

fn member_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}.{name}")
    }
}

/// Pairs `member` with `field` as one value.
///
/// Returns the kind of the value in the model, the kind of the member in the
/// CDR layout and the kind of the field in the flatbuffer layout. Fails when
/// the member and the field do not hold compatible values.
fn kinds(
    member: &MessageMember,
    field: reflection::Field<'_>,
) -> Result<(Kind, cdr::MemberKind, flatbuffer::FieldKind), Refusal> {
    let origin = origin!("compile::kinds");

    // Refuse the types that no flatbuffer field carries.
    if matches!(
        member.type_id_,
        introspection::WSTRING | introspection::WCHAR | introspection::LONG_DOUBLE
    ) {
        fail!(
            from origin,
            with Refusal::Unsupported { member: introspection::member_type(member) },
            "A {} member has no flatbuffer field that carries it", introspection::member_type(member)
        );
    }

    if member.is_array_ || member.type_id_ == introspection::MESSAGE {
        fail!(
            from origin,
            with Refusal::Unsupported { member: introspection::member_type(member) },
            "A {} member has no flatbuffer field that carries it", introspection::member_type(member)
        );
    }
    if member.type_id_ == introspection::STRING && field.type_().base_type() == BaseType::String {
        return Ok((
            Kind::String,
            cdr::MemberKind::String,
            flatbuffer::FieldKind::String,
        ));
    }
    match (
        introspection::scalar_type(member),
        schema::scalar_type(field),
    ) {
        (Some(member_type), Some(field_type)) if member_type == field_type => Ok((
            Kind::Scalar(field_type),
            cdr::MemberKind::Scalar(member_type),
            flatbuffer::FieldKind::Scalar {
                scalar: field_type,
                default: schema::default_value(field, field_type),
            },
        )),
        _ => {
            fail!(
                from origin,
                with Refusal::Mismatch { member: introspection::member_type(member), field: schema::field_type(field) },
                "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
            );
        }
    }
}
