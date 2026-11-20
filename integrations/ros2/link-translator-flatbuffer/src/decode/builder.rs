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
    Allocator, FlatBufferBuilder, Push, PushAlignment, UOffsetT, UnionWIPOffset, VOffsetT,
    WIPOffset,
};
use iceoryx2_integrations_ros2_link_adapter::cdr::reader::Endianness;

use crate::pairing::{Scalar, Value};

const SIZE_UOFFSET: usize = size_of::<UOffsetT>();

/// Pushes the scalar `value` into `slot`.
pub(super) fn push_value<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    slot: VOffsetT,
    value: Value,
) {
    match value {
        Value::Bool(value) => builder.push_slot_always(slot, value),
        Value::U8(value) => builder.push_slot_always(slot, value),
        Value::I8(value) => builder.push_slot_always(slot, value),
        Value::U16(value) => builder.push_slot_always(slot, value),
        Value::I16(value) => builder.push_slot_always(slot, value),
        Value::U32(value) => builder.push_slot_always(slot, value),
        Value::I32(value) => builder.push_slot_always(slot, value),
        Value::U64(value) => builder.push_slot_always(slot, value),
        Value::I64(value) => builder.push_slot_always(slot, value),
        Value::F32(value) => builder.push_slot_always(slot, value),
        Value::F64(value) => builder.push_slot_always(slot, value),
    }
}

/// Pushes the vector of the scalars in `bytes`, read in `endianness`.
pub(super) fn push_scalar_vector<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    scalar: Scalar,
    bytes: &[u8],
    endianness: Endianness,
) -> WIPOffset<UnionWIPOffset> {
    macro_rules! vector {
        ($ty:ty) => {{
            let elements = bytes
                .chunks_exact(core::mem::size_of::<$ty>())
                .map(|bytes| bytes.try_into().expect("the chunk is the scalar's size"))
                .map(|bytes| match endianness {
                    Endianness::Little => <$ty>::from_le_bytes(bytes),
                    Endianness::Big => <$ty>::from_be_bytes(bytes),
                });
            builder.create_vector_from_iter(elements).as_union_value()
        }};
    }
    match scalar {
        Scalar::Bool | Scalar::U8 => builder.create_vector(bytes).as_union_value(),
        Scalar::I8 => builder
            .create_vector_from_iter(bytes.iter().map(|byte| *byte as i8))
            .as_union_value(),
        Scalar::U16 => vector!(u16),
        Scalar::I16 => vector!(i16),
        Scalar::U32 => vector!(u32),
        Scalar::I32 => vector!(i32),
        Scalar::U64 => vector!(u64),
        Scalar::I64 => vector!(i64),
        Scalar::F32 => vector!(f32),
        Scalar::F64 => vector!(f64),
    }
}

/// Pushes the struct `image` into `slot`.
///
/// The builder sizes a push by its type, so an image sized by the schema
/// goes in byte by byte, in reverse, the first byte through the slot so
/// that the slot records the struct's start.
pub(super) fn push_struct<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    slot: VOffsetT,
    image: &[u8],
    alignment: usize,
) {
    align(builder, image.len(), alignment);
    let (first, rest) = image.split_first().expect("a struct has at least one byte");
    for byte in rest.iter().rev() {
        builder.push(*byte);
    }
    builder.push_slot_always(slot, *first);
}

/// Pushes the vector of the `len` struct `images`, as [`push_struct`]
/// does, with the length prefix a vector carries.
pub(super) fn push_struct_vector<A: Allocator>(
    builder: &mut FlatBufferBuilder<'_, A>,
    images: &[u8],
    len: usize,
    alignment: usize,
) -> WIPOffset<UnionWIPOffset> {
    align(builder, images.len(), alignment.max(SIZE_UOFFSET));
    for byte in images.iter().rev() {
        builder.push(*byte);
    }
    let length = builder.push(len as UOffsetT);
    WIPOffset::new(length.value())
}

/// Pads the builder so that the `number_of_bytes` pushed next start at a
/// multiple of `alignment`, which the finished flatbuffer then honors.
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
