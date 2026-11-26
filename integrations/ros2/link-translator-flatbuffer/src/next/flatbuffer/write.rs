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
    Allocator, FlatBufferBuilder, TableFinishedWIPOffset, UnionWIPOffset, VOffsetT, WIPOffset,
};
use iceoryx2_log::{fail, origin};

use super::layout::{FieldKind, FieldPlacement, Layout, TableIndex};
use crate::next::model::{ScalarValue, Values};

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

    // A list that holds the offsets of one table at a time. It is created
    // here so that writing each table reuses its memory to save reallocation.
    let mut value_offsets = Vec::new();

    // Write the tables of the layout.
    let mut written: Vec<Option<WIPOffset<TableFinishedWIPOffset>>> =
        vec![None; layout.tables.len()];
    for index in (0..layout.tables.len()).rev() {
        let table = fail!(
            from origin,
            when write_table(layout, TableIndex(index as u16), &mut value_offsets, values, builder),
            "Unable to write the table {}", index
        );
        written[index] = Some(table);
    }

    Ok(written[0].expect("a layout has a root table"))
}

fn write_table<V: Values, A: Allocator>(
    layout: &Layout,
    table: TableIndex,
    value_offsets: &mut Vec<(VOffsetT, WIPOffset<UnionWIPOffset>)>,
    values: &V,
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<WIPOffset<TableFinishedWIPOffset>, V::Error> {
    let origin = origin!("flatbuffer::write_table");

    // Write the strings of this table into the flatbuffer and save their
    // offsets.
    value_offsets.clear();
    for index in &layout.table(table).values {
        let field = layout.field(*index);
        let FieldPlacement::Table(slot) = field.placement;
        match field.kind {
            FieldKind::Scalar { .. } => {}
            FieldKind::String => {
                let bytes = fail!(
                    from origin,
                    when values.string(*index),
                    "Unable to take the string value {}", index.0
                );
                let string_offset = builder.create_byte_string(bytes).as_union_value();
                value_offsets.push((slot.vtable_offset, string_offset));
            }
        }
    }

    // Write this table. Everything from here to its end sits inside it.
    let start = builder.start_table();

    // Refer to the strings written before by their offsets.
    for (vtable_offset, value_offset) in value_offsets.iter() {
        builder.push_slot_always(*vtable_offset, *value_offset);
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
