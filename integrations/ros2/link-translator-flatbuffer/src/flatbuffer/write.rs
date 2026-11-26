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

use flatbuffers::{
    Allocator, FlatBufferBuilder, Push, PushAlignment, TableFinishedWIPOffset, UnionWIPOffset,
    VOffsetT, WIPOffset,
};
use iceoryx2_log::{fail, origin};

use super::layout::{
    FieldKind, FieldPlacement, Layout, RootObject, StructEntry, TableIndex, TablePlacement,
};
use crate::model::{ByteOrder, ScalarElements, ScalarType, ScalarValue, ValueIndex, Values};

/// Writes `values` into `builder` as a finished flatbuffer.
///
/// `layout` gives the place of each value in the flatbuffer. Fails when a
/// value cannot be taken from `values`.
pub fn write<V: Values, A: Allocator>(
    layout: &Layout,
    values: &V,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<(), V::Error> {
    let origin = origin!("flatbuffer::write");

    let root = fail!(
        from origin,
        when write_message(layout, values, builder),
        "Unable to write the values into the flatbuffer"
    );
    builder.finish(root, None);

    Ok(())
}

/// Writes `values` as the tables of the message of `layout`, returning the
/// offset of its root table.
fn write_message<V: Values, A: Allocator>(
    layout: &Layout,
    values: &V,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<TableFinishedWIPOffset>, V::Error> {
    let origin = origin!("flatbuffer::write_message");

    // Lists that hold the offsets of one table at a time. They are created
    // here so that writing each table reuses their memory to save
    // reallocation.
    let mut value_offsets = Vec::new();
    let mut element_offsets = Vec::new();

    // Write the tables last to first, so that a nested table is written
    // before the table that holds it.
    let mut written: Vec<Option<WIPOffset<TableFinishedWIPOffset>>> =
        vec![None; layout.tables.len()];
    for index in (0..layout.tables.len()).rev() {
        let table = fail!(
            from origin,
            when write_table(layout, TableIndex(index as u16), &written, &mut value_offsets, &mut element_offsets, values, builder),
            "Unable to write the table {}", index
        );
        written[index] = Some(table);
    }

    Ok(written[0].expect("a layout has a root table"))
}

fn write_table<V: Values, A: Allocator>(
    layout: &Layout,
    table: TableIndex,
    written: &[Option<WIPOffset<TableFinishedWIPOffset>>],
    value_offsets: &mut Vec<(VOffsetT, WIPOffset<UnionWIPOffset>)>,
    element_offsets: &mut Vec<WIPOffset<UnionWIPOffset>>,
    values: &V,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<TableFinishedWIPOffset>, V::Error> {
    let origin = origin!("flatbuffer::write_table");

    // Write the strings and vectors of this table into the flatbuffer and
    // save their offsets.
    value_offsets.clear();
    for index in &layout.table(table).values {
        let field = layout.field(*index);
        let FieldPlacement::Table(slot) = field.placement else {
            unreachable!("a value of a table sits in one of its slots")
        };
        match field.kind {
            FieldKind::Scalar { .. } | FieldKind::ScalarArray { .. } => {}
            FieldKind::String => {
                let bytes = fail!(
                    from origin,
                    when values.string(*index),
                    "Unable to take the string value {}", index.0
                );
                let string_offset = builder.create_byte_string(bytes).as_union_value();
                value_offsets.push((slot.vtable_offset, string_offset));
            }
            FieldKind::ScalarVector(scalar_type) => {
                let elements = fail!(
                    from origin,
                    when values.scalar_elements(*index),
                    "Unable to take the scalar elements of the value {}", index.0
                );
                let vector_offset = write_scalar_vector(builder, scalar_type, elements);
                value_offsets.push((slot.vtable_offset, vector_offset));
            }
            FieldKind::StringVector => {
                let elements = fail!(
                    from origin,
                    when values.string_elements(*index),
                    "Unable to take the string elements of the value {}", index.0
                );

                // Write the strings first. The vector then refers to them by
                // their offsets.
                element_offsets.clear();
                for element in elements {
                    let bytes = fail!(
                        from origin,
                        when element,
                        "Unable to take a string element of the value {}", index.0
                    );
                    element_offsets.push(builder.create_byte_string(bytes).as_union_value());
                }
                let vector_offset = builder
                    .create_vector(element_offsets.as_slice())
                    .as_union_value();
                value_offsets.push((slot.vtable_offset, vector_offset));
            }
            FieldKind::TableVector(model) => {
                let elements = fail!(
                    from origin,
                    when values.message_elements(*index),
                    "Unable to take the message elements of the value {}", index.0
                );

                // Write the table of each element first. The vector then
                // refers to them by their offsets.
                element_offsets.clear();
                for element in elements {
                    let element = fail!(
                        from origin,
                        when element,
                        "Unable to take a message element of the value {}", index.0
                    );
                    let element_offset = fail!(
                        from origin,
                        when write_message(layout.repeated(model), &element, builder),
                        "Unable to write a message element of the value {}", index.0
                    );
                    element_offsets.push(element_offset.as_union_value());
                }
                let vector_offset = builder
                    .create_vector(element_offsets.as_slice())
                    .as_union_value();
                value_offsets.push((slot.vtable_offset, vector_offset));
            }
            FieldKind::StructVector(model) => {
                let elements = fail!(
                    from origin,
                    when values.message_elements(*index),
                    "Unable to take the message elements of the value {}", index.0
                );
                let vector_offset = fail!(
                    from origin,
                    when write_struct_vector(layout.repeated(model), elements, builder),
                    "Unable to write the vector of structs of the value {}", index.0
                );
                value_offsets.push((slot.vtable_offset, vector_offset));
            }
        }
    }

    // Collect the offsets of the tables nested in this table.
    for (nested, table_offset) in layout.tables.iter().zip(written) {
        let TablePlacement::Nested(slot) = nested.placement else {
            continue;
        };
        if slot.table == table {
            let table_offset =
                table_offset.expect("a nested table is written before the table that holds it");
            value_offsets.push((slot.vtable_offset, table_offset.as_union_value()));
        }
    }

    // Write this table. Everything from here to its end sits inside it.
    let start = builder.start_table();

    // Refer to the strings, vectors and nested tables written before by their
    // offsets.
    for (vtable_offset, value_offset) in value_offsets.iter() {
        builder.push_slot_always(*vtable_offset, *value_offset);
    }

    // Write the structs of this table into their slots.
    for entry in &layout.structs {
        if entry.slot.table == table {
            fail!(
                from origin,
                when write_struct(layout, entry, values, builder),
                "Unable to write the struct in the slot {}", entry.slot.vtable_offset
            );
        }
    }

    // Write the scalars of this table into their slots. A scalar at its
    // default is omitted.
    for index in &layout.table(table).values {
        let field = layout.field(*index);
        if let (FieldKind::Scalar { default, .. }, FieldPlacement::Table(slot)) =
            (field.kind, field.placement)
        {
            let value = fail!(
                from origin,
                when values.scalar(*index),
                "Unable to take the scalar value {}", index.0
            );
            if value != default {
                write_scalar(builder, slot.vtable_offset, value);
            }
        }
    }

    Ok(builder.end_table(start))
}

fn write_scalar<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    vtable_offset: VOffsetT,
    value: ScalarValue,
) {
    match value {
        ScalarValue::Bool(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::U8(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::I8(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::U16(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::I16(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::U32(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::I32(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::U64(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::I64(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::F32(value) => builder.push_slot_always(vtable_offset, value),
        ScalarValue::F64(value) => builder.push_slot_always(vtable_offset, value),
    }
}

/// Writes `elements` of the type `scalar_type` as a vector, returning its offset.
fn write_scalar_vector<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    scalar_type: ScalarType,
    elements: ScalarElements<'_>,
) -> WIPOffset<UnionWIPOffset> {
    macro_rules! vector {
        ($ty:ty) => {{
            // Copy the elements as-is if they are in the byte order of the
            // host at an address aligned to their type.
            if elements.byte_order == ByteOrder::HOST {
                // SAFETY: every bit pattern is a value of the type.
                let (before, typed, after) = unsafe { elements.bytes.align_to::<$ty>() };
                if before.is_empty() && after.is_empty() {
                    return builder.create_vector(typed).as_union_value();
                }
            }

            // Push the elements one by one otherwise.
            let typed = elements.bytes.chunks_exact(size_of::<$ty>()).map(|bytes| {
                let bytes = bytes.try_into().expect("a chunk holds one element");
                match elements.byte_order {
                    ByteOrder::Little => <$ty>::from_le_bytes(bytes),
                    ByteOrder::Big => <$ty>::from_be_bytes(bytes),
                }
            });
            builder.create_vector_from_iter(typed).as_union_value()
        }};
    }

    match scalar_type {
        ScalarType::Bool | ScalarType::U8 => builder.create_vector(elements.bytes).as_union_value(),
        ScalarType::I8 => vector!(i8),
        ScalarType::U16 => vector!(u16),
        ScalarType::I16 => vector!(i16),
        ScalarType::U32 => vector!(u32),
        ScalarType::I32 => vector!(i32),
        ScalarType::U64 => vector!(u64),
        ScalarType::I64 => vector!(i64),
        ScalarType::F32 => vector!(f32),
        ScalarType::F64 => vector!(f64),
    }
}

/// Writes `elements` as a vector of the struct that is the root of `layout`,
/// returning its offset.
fn write_struct_vector<E: Values, A: Allocator>(
    layout: &Layout,
    elements: impl ExactSizeIterator<Item = Result<E, E::Error>>,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<UnionWIPOffset>, E::Error> {
    let origin = origin!("flatbuffer::write_struct_vector");

    let RootObject::Struct {
        number_of_bytes,
        alignment,
        values: struct_values,
    } = &layout.root
    else {
        unreachable!("the elements of a vector of structs are structs")
    };

    // Take every element before writing one. The builder fills the flatbuffer
    // back to front and needs the last element first.
    let mut taken_elements = Vec::with_capacity(elements.len());
    for element in elements {
        let element = fail!(
            from origin,
            when element,
            "Unable to take an element of a vector of structs"
        );
        taken_elements.push(element);
    }

    // Write the structs one after the other, aligned as a whole.
    let number_of_vector_bytes = taken_elements.len() * usize::from(*number_of_bytes);
    builder.start_vector::<u8>(number_of_vector_bytes);
    align(builder, number_of_vector_bytes, usize::from(*alignment));
    for element in taken_elements.iter().rev() {
        fail!(
            from origin,
            when push_struct(layout, struct_values, *number_of_bytes, StructDestination::Vector, element, builder),
            "Unable to write an element of a vector of structs"
        );
    }

    Ok(builder
        .end_vector::<u8>(taken_elements.len())
        .as_union_value())
}

/// Writes the struct `entry` whole into its slot of the open table.
fn write_struct<V: Values, A: Allocator>(
    layout: &Layout,
    entry: &StructEntry,
    values: &V,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<(), V::Error> {
    align(
        builder,
        usize::from(entry.number_of_bytes),
        usize::from(entry.alignment),
    );
    push_struct(
        layout,
        &entry.values,
        entry.number_of_bytes,
        StructDestination::Slot(entry.slot.vtable_offset),
        values,
        builder,
    )
}

/// Where a struct that is pushed into the builder should be placed.
#[derive(Debug, Clone, Copy)]
enum StructDestination {
    /// A slot of the open table.
    Slot(VOffsetT),
    /// The open vector.
    Vector,
}

/// Pushes the struct of `number_of_bytes` that holds `struct_values`, which
/// are in the order of their offsets.
fn push_struct<V: Values, A: Allocator>(
    layout: &Layout,
    struct_values: &[ValueIndex],
    number_of_bytes: u16,
    destination: StructDestination,
    values: &V,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<(), V::Error> {
    let origin = origin!("flatbuffer::push_struct");

    // The builder fills the flatbuffer back to front. Push the values last
    // to first with the padding that follows each.
    let mut position = usize::from(number_of_bytes);
    for index in struct_values.iter().rev() {
        let field = layout.field(*index);
        let (FieldPlacement::Struct { offset, .. } | FieldPlacement::VectorElement { offset }) =
            field.placement
        else {
            unreachable!("the value sits in the struct")
        };
        match field.kind {
            FieldKind::Scalar { scalar_type, .. } => {
                let value = fail!(
                    from origin,
                    when values.scalar(*index),
                    "Unable to take the scalar value {}", index.0
                );

                let bytes = &value.to_le_bytes()[..scalar_type.number_of_bytes()];
                let end = usize::from(offset.0) + bytes.len();
                push_padding(builder, destination, &mut position, end);
                push_bytes(builder, destination, &mut position, bytes);
            }
            FieldKind::ScalarArray {
                scalar_type,
                number_of_elements,
            } => {
                let elements = fail!(
                    from origin,
                    when values.scalar_elements(*index),
                    "Unable to take the scalar elements of the value {}", index.0
                );

                let number_of_bytes = number_of_elements as usize * scalar_type.number_of_bytes();
                let end = usize::from(offset.0) + number_of_bytes;
                push_padding(builder, destination, &mut position, end);

                // Left zero if no elements to push.
                if elements.bytes.len() == number_of_bytes {
                    push_scalar_elements(
                        builder,
                        destination,
                        &mut position,
                        scalar_type,
                        elements,
                    );
                }
            }
            FieldKind::String
            | FieldKind::ScalarVector(_)
            | FieldKind::StringVector
            | FieldKind::TableVector(_)
            | FieldKind::StructVector(_) => {
                unreachable!("a struct holds scalars and arrays of scalars")
            }
        }
    }

    // Push the padding before the first value.
    push_padding(builder, destination, &mut position, 0);

    Ok(())
}

/// Pushes `elements` of the type `scalar_type` into a struct last to first from
/// `position` back.
fn push_scalar_elements<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    destination: StructDestination,
    position: &mut usize,
    scalar_type: ScalarType,
    elements: ScalarElements<'_>,
) {
    if elements.byte_order == ByteOrder::Little || scalar_type.number_of_bytes() == 1 {
        // The elements lie in the byte order of the flatbuffer.
        push_bytes(builder, destination, position, elements.bytes);
    } else {
        // The elements lie in the other byte order.
        for bytes in elements
            .bytes
            .chunks_exact(scalar_type.number_of_bytes())
            .rev()
        {
            let value = ScalarValue::from_bytes(scalar_type, bytes, elements.byte_order);
            push_bytes(
                builder,
                destination,
                position,
                &value.to_le_bytes()[..scalar_type.number_of_bytes()],
            );
        }
    }
}

/// Pushes zeros into a struct from `position` back to `end`.
fn push_padding<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    destination: StructDestination,
    position: &mut usize,
    end: usize,
) {
    while *position > end {
        push_byte(builder, destination, position, 0);
    }
}

/// Pushes `bytes` into a struct last to first from `position` back.
fn push_bytes<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    destination: StructDestination,
    position: &mut usize,
    bytes: &[u8],
) {
    for byte in bytes.iter().rev() {
        push_byte(builder, destination, position, *byte);
    }
}

/// Pushes the byte before `position` into a struct.
fn push_byte<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    destination: StructDestination,
    position: &mut usize,
    byte: u8,
) {
    *position -= 1;
    match (*position, destination) {
        // The first byte of a struct in a slot is pushed through the slot,
        // which then refers to the start of the struct.
        (0, StructDestination::Slot(vtable_offset)) => {
            builder.push_slot_always(vtable_offset, byte)
        }
        _ => {
            builder.push(byte);
        }
    }
}

/// Pads the builder so that the `number_of_bytes` pushed next start at a
/// multiple of `alignment` in the finished flatbuffer.
fn align<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    number_of_bytes: usize,
    alignment: usize,
) {
    match alignment {
        1 => {
            builder.push(Align::<1>);
        }
        2 => {
            builder.push(Align::<2>);
        }
        4 => {
            builder.push(Align::<4>);
        }
        8 => {
            builder.push(Align::<8>);
        }
        16 => {
            builder.push(Align::<16>);
        }
        other => unreachable!("a flatbuffer aligns to at most 16 bytes, not {other}"),
    }
    while !(builder.unfinished_data().len() + number_of_bytes).is_multiple_of(alignment) {
        builder.push(0u8);
    }
}

/// A push of no bytes that registers an alignment with the builder.
struct Align<const N: usize>;

impl<const N: usize> Push for Align<N> {
    type Output = Self;

    unsafe fn push(&self, _dst: &mut [u8], _written_len: usize) {}

    fn size() -> usize {
        0
    }

    fn alignment() -> PushAlignment {
        PushAlignment::new(N)
    }
}
