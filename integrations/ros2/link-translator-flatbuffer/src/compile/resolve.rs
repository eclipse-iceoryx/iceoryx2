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

use iceoryx2_bb_flatbuffers::reflection::{self, BaseType, Object};
use iceoryx2_log::{fail, origin};
use r2r_rcl::rosidl_typesupport_introspection_c__MessageMember as MessageMember;

use super::pairing::Pair;
use super::{Refusal, introspection, schema};
use crate::flatbuffer::FieldPlacement;
use crate::model::Kind;
use crate::{cdr, flatbuffer};

/// What a member and the field it is paired with hold together.
pub(super) enum Resolved<'a> {
    /// One value of the model.
    Value {
        kind: Kind,
        member_kind: cdr::MemberKind,
        field_kind: flatbuffer::FieldKind,
    },
    /// A nested message that the table or the struct `object` holds.
    Nested { object: Object<'a> },
    /// A sequence or an array of messages that a vector of the table or the
    /// struct `element` holds.
    Repeated {
        element: Object<'a>,
        /// None for a sequence.
        number_of_elements: Option<u32>,
    },
}

/// Resolves `pair` against the `field` that it is placed at.
///
/// Fails when the member and the field do not hold compatible values.
pub(super) fn pair<'a>(
    schema: reflection::Schema<'a>,
    pair: &Pair<'_>,
    field: reflection::Field<'_>,
    placement: FieldPlacement,
) -> Result<Resolved<'a>, Refusal> {
    match placement {
        FieldPlacement::Table(_) => table_field(schema, pair, field),
        FieldPlacement::Struct { .. } | FieldPlacement::VectorElement { .. } => {
            struct_field(schema, pair, field)
        }
    }
}

fn table_field<'a>(
    schema: reflection::Schema<'a>,
    pair: &Pair<'_>,
    field: reflection::Field<'_>,
) -> Result<Resolved<'a>, Refusal> {
    let origin = origin!("compile::resolve::table_field");

    let member = pair.member;

    if member.type_id_ == introspection::MESSAGE && member.is_array_ {
        let Some(element) = schema::element_object(schema, field) else {
            fail!(
                from origin,
                with mismatch(member, field),
                "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
            );
        };
        let number_of_elements = if introspection::is_sequence(member) {
            None
        } else {
            Some(member.array_size_ as u32)
        };

        return Ok(Resolved::Repeated {
            element,
            number_of_elements,
        });
    }

    if member.type_id_ == introspection::MESSAGE {
        let Some(object) = schema::object(schema, field) else {
            fail!(
                from origin,
                with mismatch(member, field),
                "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
            );
        };

        return Ok(Resolved::Nested { object });
    }

    value(pair, field)
}

fn struct_field<'a>(
    schema: reflection::Schema<'a>,
    pair: &Pair<'_>,
    field: reflection::Field<'_>,
) -> Result<Resolved<'a>, Refusal> {
    let origin = origin!("compile::resolve::struct_field");

    let member = pair.member;

    // A struct holds a nested message only as a nested struct.
    if member.type_id_ == introspection::MESSAGE && !member.is_array_ {
        return match schema::object(schema, field) {
            Some(object) if object.is_struct() => Ok(Resolved::Nested { object }),
            _ => {
                fail!(
                    from origin,
                    with mismatch(member, field),
                    "A {} member does not pair with a {} field", introspection::member_type(member), schema::field_type(field)
                );
            }
        };
    }

    value(pair, field)
}

fn value<'a>(pair: &Pair<'_>, field: reflection::Field<'_>) -> Result<Resolved<'a>, Refusal> {
    let (kind, member_kind, field_kind) = kinds(pair.member, field)?;

    Ok(Resolved::Value {
        kind,
        member_kind,
        field_kind,
    })
}

/// The kind of the value in the model, the kind of the member in the CDR
/// layout and the kind of the field in the flatbuffer layout.
///
/// Fails when the member and the field do not hold compatible values.
fn kinds(
    member: &MessageMember,
    field: reflection::Field<'_>,
) -> Result<(Kind, cdr::MemberKind, flatbuffer::FieldKind), Refusal> {
    let origin = origin!("compile::resolve::kinds");

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

fn mismatch(member: &MessageMember, field: reflection::Field<'_>) -> Refusal {
    Refusal::Mismatch {
        member: introspection::member_type(member),
        field: schema::field_type(field),
    }
}
