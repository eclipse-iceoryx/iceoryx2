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
mod pairing;
mod resolve;
mod schema;

use iceoryx2_bb_flatbuffers::reflection::{self, Object, root_as_schema};
use iceoryx2_integrations_ros2_link_adapter::typesupport::{IntrospectionKind, TypeSupport};
use iceoryx2_link_backend::service_description::Schema;
use iceoryx2_log::{fail, origin};
use r2r_rcl::{
    rosidl_typesupport_introspection_c__MessageMember as MessageMember,
    rosidl_typesupport_introspection_c__MessageMembers as MessageMembers,
};

use super::flatbuffer::{FieldPlacement, Slot, StructOffset, TableIndex, TablePlacement};
use super::model::{Kind, Model, ModelIndex, ValueIndex};
use super::{cdr, flatbuffer};
use pairing::Pair;
use resolve::Resolved;

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
    MissingField,
    Mismatch { member: String, field: String },
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
    const ROOT_MESSAGE_PATH: &str = "";

    let origin = origin!("compile");

    // Find the root table of the schema.
    let schema = root_as_schema(schema.bytes()).expect("a schema holds a binary flatbuffer schema");
    let root = schema.root_table().expect("a schema declares a root type");

    // Walk the message and the root table together.
    let mut compiler = Compiler {
        schema,
        compiled: Compiled::default(),
    };

    // SAFETY: the typesupport keeps the members valid while it is loaded.
    if let Err(error) = unsafe {
        compiler.compile_table(
            root,
            typesupport.members(),
            TablePlacement::Root,
            ROOT_MESSAGE_PATH,
        )
    } {
        fail!(
            from origin,
            with error,
            "Member '{}' has no value in '{}'", error.path, root.name()
        );
    }

    Ok(compiler.compiled)
}

struct Compiler<'a> {
    schema: reflection::Schema<'a>,
    compiled: Compiled,
}

impl<'a> Compiler<'a> {
    /// Compiles a message that a table holds.
    ///
    /// # Safety
    ///
    /// The introspection typesupport that `message` points into must be loaded.
    unsafe fn compile_table(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        placement: TablePlacement,
        message_path: &str,
    ) -> Result<(), CompileError> {
        let table = self.compiled.flatbuffer.add_table(placement);
        unsafe { self.compile_table_fields(object, table, message, message_path)? };
        self.compiled.flatbuffer.sort_struct_values();

        Ok(())
    }

    /// Compiles the fields of the table `object` with the members of a message.
    ///
    /// # Safety
    ///
    /// The introspection typesupport that `message` points into must be loaded.
    unsafe fn compile_table_fields(
        &mut self,
        object: Object<'a>,
        table: TableIndex,
        message: *const MessageMembers,
        message_path: &str,
    ) -> Result<(), CompileError> {
        for pair in unsafe { pairing::pair_members_with_fields(message, message_path) } {
            let (placement, field) = self
                .place_table_field(object, table, pair.field_path)
                .map_err(|refusal| CompileError {
                    path: pair.member_path.clone(),
                    refusal,
                })?;
            match placement {
                FieldPlacement::Table(slot) => unsafe {
                    self.compile_table_field(pair, field, slot)?
                },
                FieldPlacement::Struct { .. } | FieldPlacement::VectorElement { .. } => {
                    unreachable!("a field of a table sits in a slot")
                }
            }
        }

        Ok(())
    }

    /// Compiles the table field `field` with the member it is paired with.
    ///
    /// # Safety
    ///
    /// The introspection typesupport of the member of `pair` must be loaded.
    unsafe fn compile_table_field(
        &mut self,
        pair: Pair<'_>,
        field: reflection::Field<'a>,
        slot: Slot,
    ) -> Result<(), CompileError> {
        let placement = FieldPlacement::Table(slot);
        let resolved = resolve::pair(self.schema, &pair, field, placement).map_err(|refusal| {
            CompileError {
                path: pair.member_path.clone(),
                refusal,
            }
        })?;
        let Pair {
            member,
            member_path,
            ..
        } = pair;
        let (kind, member_kind, field_kind) = match resolved {
            Resolved::Value {
                kind,
                member_kind,
                field_kind,
            } => (kind, member_kind, field_kind),
            Resolved::Nested { object: nested } => {
                return unsafe { self.compile_nested(member, nested, slot, &member_path) };
            }
            Resolved::Repeated {
                element,
                number_of_elements,
            } => unsafe {
                self.compile_repeated(member, element, number_of_elements, &member_path)?
            },
        };

        let index = self
            .compiled
            .add_value(kind, member_kind, placement, field_kind);
        self.compiled.flatbuffer.add_table_value(slot, index);

        Ok(())
    }

    /// Compiles a nested message that a struct in a slot of a table holds.
    ///
    /// # Safety
    ///
    /// The introspection typesupport that `message` points into must be loaded.
    unsafe fn compile_struct(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        slot: Slot,
        message_path: &str,
    ) -> Result<(), CompileError> {
        self.compiled.flatbuffer.add_struct(
            slot,
            object.bytesize() as u16,
            object.minalign() as u16,
        );

        let start = FieldPlacement::Struct {
            slot,
            offset: START_OF_STRUCT,
        };
        unsafe { self.compile_struct_fields(object, start, message, message_path) }
    }

    /// Compiles the fields of the struct `object` with the members of a message.
    ///
    /// # Safety
    ///
    /// The introspection typesupport that `message` points into must be loaded.
    unsafe fn compile_struct_fields(
        &mut self,
        object: Object<'a>,
        start: FieldPlacement,
        message: *const MessageMembers,
        message_path: &str,
    ) -> Result<(), CompileError> {
        for pair in unsafe { pairing::pair_members_with_fields(message, message_path) } {
            let (placement, field) = self
                .place_struct_field(object, start, pair.field_path)
                .map_err(|refusal| CompileError {
                    path: pair.member_path.clone(),
                    refusal,
                })?;
            unsafe { self.compile_struct_field(pair, field, placement)? };
        }

        Ok(())
    }

    /// Compiles the struct field `field` with the member it is paired with.
    ///
    /// # Safety
    ///
    /// The introspection typesupport of the member of `pair` must be loaded.
    unsafe fn compile_struct_field(
        &mut self,
        pair: Pair<'_>,
        field: reflection::Field<'a>,
        placement: FieldPlacement,
    ) -> Result<(), CompileError> {
        let resolved = resolve::pair(self.schema, &pair, field, placement).map_err(|refusal| {
            CompileError {
                path: pair.member_path.clone(),
                refusal,
            }
        })?;
        let Pair {
            member,
            member_path,
            ..
        } = pair;

        let (kind, member_kind, field_kind) = match resolved {
            Resolved::Value {
                kind,
                member_kind,
                field_kind,
            } => (kind, member_kind, field_kind),

            // Add the values of a nested message that a nested struct holds.
            // The nested struct starts where the field is placed.
            Resolved::Nested { object: nested } => {
                let message = unsafe { introspection::message(member) };
                return unsafe {
                    self.compile_struct_fields(nested, placement, message, &member_path)
                };
            }

            Resolved::Repeated { .. } => unreachable!("a struct holds no sequence of messages"),
        };
        let index = self
            .compiled
            .add_value(kind, member_kind, placement, field_kind);
        self.compiled.flatbuffer.add_struct_value(placement, index);

        Ok(())
    }

    /// Compiles the nested message of `member` that the table or the struct
    /// `nested` in `slot` holds.
    ///
    /// # Safety
    ///
    /// The introspection typesupport of `member` must be loaded.
    unsafe fn compile_nested(
        &mut self,
        member: &MessageMember,
        nested: Object<'a>,
        slot: Slot,
        member_path: &str,
    ) -> Result<(), CompileError> {
        let message = unsafe { introspection::message(member) };
        if nested.is_struct() {
            unsafe { self.compile_struct(nested, message, slot, member_path) }
        } else {
            let placement = TablePlacement::Nested(slot);
            unsafe { self.compile_table(nested, message, placement, member_path) }
        }
    }

    /// Compiles the message that `member` repeats and that a vector of the
    /// table or the struct `element` holds.
    ///
    /// # Safety
    ///
    /// The introspection typesupport of `member` must be loaded.
    unsafe fn compile_repeated(
        &mut self,
        member: &MessageMember,
        element: Object<'a>,
        number_of_elements: Option<u32>,
        member_path: &str,
    ) -> Result<(Kind, cdr::MemberKind, flatbuffer::FieldKind), CompileError> {
        let message = unsafe { introspection::message(member) };
        let (model, field_kind) = if element.is_struct() {
            let model = unsafe { self.compile_repeated_struct(element, message, member_path)? };
            (model, flatbuffer::FieldKind::StructVector(model))
        } else {
            let model = unsafe { self.compile_repeated_table(element, message, member_path)? };
            (model, flatbuffer::FieldKind::TableVector(model))
        };
        let (kind, member_kind) = match number_of_elements {
            None => (
                Kind::MessageSequence(model),
                cdr::MemberKind::MessageSequence(model),
            ),
            Some(number_of_elements) => (
                Kind::MessageArray {
                    model,
                    number_of_elements,
                },
                cdr::MemberKind::MessageArray {
                    model,
                    number_of_elements,
                },
            ),
        };

        Ok((kind, member_kind, field_kind))
    }

    /// Compiles a message that a table holds as the element of a sequence.
    ///
    /// # Safety
    ///
    /// The introspection typesupport that `message` points into must be loaded.
    unsafe fn compile_repeated_table(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        message_path: &str,
    ) -> Result<ModelIndex, CompileError> {
        // Compile the message like a root message.
        let mut compiler = Compiler {
            schema: self.schema,
            compiled: Compiled::default(),
        };
        unsafe { compiler.compile_table(object, message, TablePlacement::Root, message_path)? };

        Ok(self.compiled.add_repeated(compiler.compiled))
    }

    /// Compiles a message that a struct holds as the element of a sequence.
    ///
    /// # Safety
    ///
    /// The introspection typesupport that `message` points into must be loaded.
    unsafe fn compile_repeated_struct(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        message_path: &str,
    ) -> Result<ModelIndex, CompileError> {
        // Compile the message with the struct as the root of its layout.
        let mut compiler = Compiler {
            schema: self.schema,
            compiled: Compiled::default(),
        };
        compiler.compiled.flatbuffer.root = flatbuffer::RootObject::Struct {
            number_of_bytes: object.bytesize() as u16,
            alignment: object.minalign() as u16,
            values: Vec::new(),
        };
        let start = FieldPlacement::VectorElement {
            offset: START_OF_STRUCT,
        };
        unsafe { compiler.compile_struct_fields(object, start, message, message_path)? };

        compiler.compiled.flatbuffer.sort_struct_values();

        Ok(self.compiled.add_repeated(compiler.compiled))
    }

    /// Places the field `field_path` of the table `object`.
    ///
    /// Fails when the table has no such field.
    fn place_table_field(
        &self,
        object: Object<'a>,
        table: TableIndex,
        field_path: &str,
    ) -> Result<(FieldPlacement, reflection::Field<'a>), Refusal> {
        let field = schema::field(object, field_path)?;
        let slot = Slot {
            table,
            vtable_offset: field.offset(),
        };

        Ok((FieldPlacement::Table(slot), field))
    }

    /// Places the field `field_path` of the struct `object`.
    ///
    /// `start` is the placement of the first byte of `object`. Fails when the
    /// struct has no such field.
    fn place_struct_field(
        &self,
        object: Object<'a>,
        start: FieldPlacement,
        field_path: &str,
    ) -> Result<(FieldPlacement, reflection::Field<'a>), Refusal> {
        let field = schema::field(object, field_path)?;
        let placement = match start {
            FieldPlacement::Struct { slot, offset } => FieldPlacement::Struct {
                slot,
                offset: StructOffset(offset.0 + field.offset()),
            },
            FieldPlacement::VectorElement { offset } => FieldPlacement::VectorElement {
                offset: StructOffset(offset.0 + field.offset()),
            },
            FieldPlacement::Table(_) => unreachable!("a struct starts in a struct"),
        };

        Ok((placement, field))
    }
}

impl Compiled {
    /// Adds a value of `kind` with its member and its field.
    ///
    /// Returns the index of the value.
    fn add_value(
        &mut self,
        kind: Kind,
        member_kind: cdr::MemberKind,
        placement: FieldPlacement,
        field_kind: flatbuffer::FieldKind,
    ) -> ValueIndex {
        // Add the value to the model.
        let index = ValueIndex(self.model.values.len() as u16);
        self.model.values.push(kind);

        // Add the member of the value to the CDR layout.
        self.cdr
            .members
            .push(cdr::MemberEntry { kind: member_kind });
        self.cdr.order.push(index);

        // Add the field of the value to the flatbuffer layout.
        self.flatbuffer.fields.push(flatbuffer::FieldEntry {
            placement,
            kind: field_kind,
        });

        index
    }

    /// Adds the compiled element of a sequence to the repeated models and
    /// layouts.
    ///
    /// Returns the index they share.
    fn add_repeated(&mut self, repeated: Compiled) -> ModelIndex {
        let index = ModelIndex(self.model.repeated.len() as u16);
        self.model.repeated.push(repeated.model);
        self.cdr.repeated.push(repeated.cdr);
        self.flatbuffer.repeated.push(repeated.flatbuffer);

        index
    }
}

/// The offset of the first byte of a struct.
const START_OF_STRUCT: StructOffset = StructOffset(0);

fn join_path(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}.{name}")
    }
}
