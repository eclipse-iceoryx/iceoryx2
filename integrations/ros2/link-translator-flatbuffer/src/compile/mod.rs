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
mod schema;

use iceoryx2_bb_flatbuffers::reflection::{self, BaseType, Object, root_as_schema};
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
use pairing::MemberPairing;

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
    if let Err(error) =
        unsafe { compiler.add_table(root, typesupport.members(), TablePlacement::Root, "") }
    {
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
    /// Adds the table that holds a message.
    ///
    /// # Safety
    ///
    /// `message` must point to the members of a loaded introspection typesupport.
    unsafe fn add_table(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        placement: TablePlacement,
        message_path: &str,
    ) -> Result<(), CompileError> {
        let origin = origin!("compile::add_table");

        // Add the table to the flatbuffer layout.
        let table = TableIndex(self.compiled.flatbuffer.tables.len() as u16);
        self.compiled
            .flatbuffer
            .tables
            .push(flatbuffer::TableEntry {
                placement,
                values: Vec::new(),
            });

        for MemberPairing {
            member,
            member_path,
            field_path,
        } in unsafe { pairing::pairings(message, message_path) }
        {
            let (placement, field) =
                self.place_table_field(object, table, field_path)
                    .map_err(|refusal| CompileError {
                        path: member_path.clone(),
                        refusal,
                    })?;
            let FieldPlacement::Table(slot) = placement else {
                unreachable!("a field of a table sits in a slot")
            };

            // A sequence or an array of messages is one value of this model.
            // The message it repeats is compiled separately, and the value
            // refers to that model.
            if member.type_id_ == introspection::MESSAGE && member.is_array_ {
                let Some(element) = schema::element_object(self.schema, field) else {
                    fail!(
                        from origin,
                        with CompileError { path: member_path, refusal: mismatch(member, field) },
                        "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
                    );
                };
                let message = unsafe { introspection::message(member) };
                let (model, field_kind) = if element.is_struct() {
                    let model =
                        unsafe { self.add_repeated_struct(element, message, &member_path)? };
                    (model, flatbuffer::FieldKind::StructVector(model))
                } else {
                    let model = unsafe { self.add_repeated_table(element, message, &member_path)? };
                    (model, flatbuffer::FieldKind::TableVector(model))
                };

                let (kind, member_kind) = if introspection::is_sequence(member) {
                    (
                        Kind::MessageSequence(model),
                        cdr::MemberKind::MessageSequence(model),
                    )
                } else {
                    let number_of_elements = member.array_size_ as u32;
                    (
                        Kind::MessageArray {
                            model,
                            number_of_elements,
                        },
                        cdr::MemberKind::MessageArray {
                            model,
                            number_of_elements,
                        },
                    )
                };
                let index = self.add_value(kind, member_kind, placement, field_kind);
                self.compiled.flatbuffer.tables[usize::from(slot.table.0)]
                    .values
                    .push(index);
                continue;
            }

            // Define a nested table or a struct at the field to hold a nested
            // message.
            if member.type_id_ == introspection::MESSAGE && !member.is_array_ {
                let Some(nested) = schema::object(self.schema, field) else {
                    fail!(
                        from origin,
                        with CompileError { path: member_path, refusal: mismatch(member, field) },
                        "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
                    );
                };
                let message = unsafe { introspection::message(member) };
                if nested.is_struct() {
                    unsafe { self.add_struct(nested, message, slot, &member_path)? };
                } else {
                    let placement = TablePlacement::Nested(slot);
                    unsafe { self.add_table(nested, message, placement, &member_path)? };
                }
                continue;
            }

            // Check that the member and the field hold the same kind of value.
            let (kind, member_kind, field_kind) =
                pair(member, field).map_err(|refusal| CompileError {
                    path: member_path,
                    refusal,
                })?;

            let index = self.add_value(kind, member_kind, placement, field_kind);
            self.compiled.flatbuffer.tables[usize::from(slot.table.0)]
                .values
                .push(index);
        }

        Ok(())
    }

    /// Adds the struct that holds a nested message in a slot of a table.
    ///
    /// # Safety
    ///
    /// `message` must point to the members of a loaded introspection typesupport.
    unsafe fn add_struct(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        slot: Slot,
        message_path: &str,
    ) -> Result<(), CompileError> {
        // Add the struct to the flatbuffer layout.
        let entry = self.compiled.flatbuffer.structs.len();
        self.compiled
            .flatbuffer
            .structs
            .push(flatbuffer::StructEntry {
                slot,
                number_of_bytes: object.bytesize() as u16,
                alignment: object.minalign() as u16,
                values: Vec::new(),
            });

        let start = FieldPlacement::Struct {
            slot,
            offset: START_OF_STRUCT,
        };
        unsafe { self.add_struct_values(object, message, start, message_path)? };

        // Order the values of the struct by their offsets.
        let layout = &mut self.compiled.flatbuffer;
        sort_by_offset(&mut layout.structs[entry].values, &layout.fields);

        Ok(())
    }

    /// Adds the values of a message to the struct that holds them.
    ///
    /// `start` is the placement of the first byte of `object`.
    ///
    /// # Safety
    ///
    /// `message` must point to the members of a loaded introspection typesupport.
    unsafe fn add_struct_values(
        &mut self,
        object: Object<'a>,
        message: *const MessageMembers,
        start: FieldPlacement,
        message_path: &str,
    ) -> Result<(), CompileError> {
        for MemberPairing {
            member,
            member_path,
            field_path,
        } in unsafe { pairing::pairings(message, message_path) }
        {
            let (placement, field) =
                self.place_struct_field(object, start, field_path)
                    .map_err(|refusal| CompileError {
                        path: member_path.clone(),
                        refusal,
                    })?;
            unsafe { self.add_struct_value(member, field, placement, member_path)? };
        }

        Ok(())
    }

    /// Adds the values of `member` to the struct that holds `field`.
    ///
    /// `placement` is the placement of `field`.
    ///
    /// # Safety
    ///
    /// `member` must belong to a loaded introspection typesupport.
    unsafe fn add_struct_value(
        &mut self,
        member: &MessageMember,
        field: reflection::Field<'a>,
        placement: FieldPlacement,
        member_path: String,
    ) -> Result<(), CompileError> {
        let origin = origin!("compile::add_struct_value");

        // Add the values of a nested message that a nested struct holds. The
        // nested struct starts where the field is placed.
        if member.type_id_ == introspection::MESSAGE && !member.is_array_ {
            let nested = match schema::object(self.schema, field) {
                Some(nested) if nested.is_struct() => nested,
                _ => {
                    fail!(
                        from origin,
                        with CompileError { path: member_path, refusal: mismatch(member, field) },
                        "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
                    );
                }
            };
            let message = unsafe { introspection::message(member) };
            return unsafe { self.add_struct_values(nested, message, placement, &member_path) };
        }

        // Check that the member and the field hold the same scalar.
        let (kind, member_kind, field_kind) =
            pair(member, field).map_err(|refusal| CompileError {
                path: member_path,
                refusal,
            })?;
        let index = self.add_value(kind, member_kind, placement, field_kind);

        // Add the value to the struct it is placed in.
        let layout = &mut self.compiled.flatbuffer;
        match (placement, &mut layout.root) {
            (FieldPlacement::Struct { slot, .. }, _) => layout
                .structs
                .iter_mut()
                .find(|entry| entry.slot == slot)
                .expect("the struct of a placement is part of the layout")
                .values
                .push(index),
            (
                FieldPlacement::VectorElement { .. },
                flatbuffer::RootObject::Struct { values, .. },
            ) => values.push(index),
            (FieldPlacement::VectorElement { .. }, flatbuffer::RootObject::Table)
            | (FieldPlacement::Table(_), _) => {
                unreachable!("the value is placed in a struct")
            }
        }

        Ok(())
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

    /// Adds the table that holds a message as the element of a sequence.
    ///
    /// # Safety
    ///
    /// `message` must point to the members of a loaded introspection typesupport.
    unsafe fn add_repeated_table(
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
        unsafe { compiler.add_table(object, message, TablePlacement::Root, message_path)? };

        Ok(self.add_repeated(compiler.compiled))
    }

    /// Adds the struct that holds a message as the element of a sequence.
    ///
    /// # Safety
    ///
    /// `message` must point to the members of a loaded introspection typesupport.
    unsafe fn add_repeated_struct(
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
        unsafe { compiler.add_struct_values(object, message, start, message_path)? };

        // Order the values of the struct by their offsets.
        let layout = &mut compiler.compiled.flatbuffer;
        if let flatbuffer::RootObject::Struct { values, .. } = &mut layout.root {
            sort_by_offset(values, &layout.fields);
        }

        Ok(self.add_repeated(compiler.compiled))
    }

    /// Adds the compiled element of a sequence to the repeated models and
    /// layouts of this message.
    ///
    /// Returns the index they share.
    fn add_repeated(&mut self, repeated: Compiled) -> ModelIndex {
        let index = ModelIndex(self.compiled.model.repeated.len() as u16);
        self.compiled.model.repeated.push(repeated.model);
        self.compiled.cdr.repeated.push(repeated.cdr);
        self.compiled.flatbuffer.repeated.push(repeated.flatbuffer);

        index
    }

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

/// The offset of the first byte of a struct.
const START_OF_STRUCT: StructOffset = StructOffset(0);

fn sort_by_offset(values: &mut [ValueIndex], fields: &[flatbuffer::FieldEntry]) {
    values.sort_by_key(|index| match fields[usize::from(index.0)].placement {
        FieldPlacement::Struct { offset, .. } | FieldPlacement::VectorElement { offset } => offset,
        FieldPlacement::Table(_) => unreachable!("the value sits in a struct"),
    });
}

fn mismatch(member: &MessageMember, field: reflection::Field<'_>) -> Refusal {
    Refusal::Mismatch {
        member: introspection::member_type(member),
        field: schema::field_type(field),
    }
}

fn join_path(parent: &str, name: &str) -> String {
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
fn pair(
    member: &MessageMember,
    field: reflection::Field<'_>,
) -> Result<(Kind, cdr::MemberKind, flatbuffer::FieldKind), Refusal> {
    let origin = origin!("compile::pair");

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

    // Pair a sequence of scalars with a vector of the same scalar.
    if introspection::is_sequence(member)
        && let Some(member_type) = introspection::scalar_type(member)
    {
        return match schema::element_scalar_type(field) {
            Some(field_type) if member_type == field_type => Ok((
                Kind::ScalarSequence(field_type),
                cdr::MemberKind::ScalarSequence(member_type),
                flatbuffer::FieldKind::ScalarVector(field_type),
            )),
            _ => {
                fail!(
                    from origin,
                    with mismatch(member, field),
                    "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
                );
            }
        };
    }

    // Pair an array of scalars with a vector of the same scalar or with an
    // array of the same scalar and number of elements.
    if introspection::is_array(member)
        && let Some(member_type) = introspection::scalar_type(member)
    {
        let number_of_elements = member.array_size_ as u32;
        let kind = Kind::ScalarArray {
            scalar_type: member_type,
            number_of_elements,
        };
        let member_kind = cdr::MemberKind::ScalarArray {
            scalar_type: member_type,
            number_of_elements,
        };
        if schema::element_scalar_type(field) == Some(member_type) {
            let field_kind = flatbuffer::FieldKind::ScalarVector(member_type);
            return Ok((kind, member_kind, field_kind));
        }
        return match schema::scalar_array(field) {
            Some(array) if array == (member_type, number_of_elements) => Ok((
                kind,
                member_kind,
                flatbuffer::FieldKind::ScalarArray {
                    scalar_type: member_type,
                    number_of_elements,
                },
            )),
            _ => {
                fail!(
                    from origin,
                    with mismatch(member, field),
                    "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
                );
            }
        };
    }

    // Pair a sequence or an array of strings with a vector of strings.
    if member.type_id_ == introspection::STRING
        && (introspection::is_sequence(member) || introspection::is_array(member))
    {
        if !schema::is_string_vector(field) {
            fail!(
                from origin,
                with mismatch(member, field),
                "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
            );
        }

        // The model and the CDR distinguish a sequence from an array. The
        // flatbuffer holds both as a vector of strings.
        let (kind, member_kind) = if introspection::is_sequence(member) {
            (Kind::StringSequence, cdr::MemberKind::StringSequence)
        } else {
            let number_of_elements = member.array_size_ as u32;
            (
                Kind::StringArray { number_of_elements },
                cdr::MemberKind::StringArray { number_of_elements },
            )
        };

        return Ok((kind, member_kind, flatbuffer::FieldKind::StringVector));
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
                scalar_type: field_type,
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
