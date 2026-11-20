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

use flatbuffers::{Follow, ForwardsUOffset, Table, UOffsetT, VOffsetT, Vector};

use crate::pairing::{Scalar, Value};

const SIZE_UOFFSET: usize = size_of::<UOffsetT>();

/// The root table of `flatbuffer`, `None` when the bytes hold no root
/// offset.
pub(super) fn root(flatbuffer: &[u8]) -> Option<Table<'_>> {
    if flatbuffer.len() < SIZE_UOFFSET {
        return None;
    }
    // SAFETY: the flatbuffer is trusted to be well-formed.
    Some(unsafe { ForwardsUOffset::<Table<'_>>::follow(flatbuffer, 0) })
}

/// The position in `table` of the field in `slot`, `None` when absent.
pub(super) fn position(table: Table<'_>, slot: VOffsetT) -> Option<usize> {
    match table.vtable().get(slot) {
        0 => None,
        offset => Some(table.loc() + offset as usize),
    }
}

/// The `scalar` at `position` of `buf`.
pub(super) fn scalar(buf: &[u8], position: usize, scalar: Scalar) -> Value {
    Value::from_le_bytes(scalar, &buf[position..])
}

/// The string at `position` of `buf`.
pub(super) fn string(buf: &[u8], position: usize) -> &[u8] {
    // SAFETY: the field pairs with a string.
    unsafe { ForwardsUOffset::<&[u8]>::follow(buf, position) }
}

/// The table at `position` of `buf`.
pub(super) fn table(buf: &[u8], position: usize) -> Table<'_> {
    // SAFETY: the field pairs with a table.
    unsafe { ForwardsUOffset::<Table<'_>>::follow(buf, position) }
}

/// The bytes from the start of the struct at `position` of `buf`.
pub(super) fn struct_bytes(buf: &[u8], position: usize) -> &[u8] {
    &buf[position..]
}

/// The vector at `position` of `buf`.
pub(super) fn vector(buf: &[u8], position: usize) -> RawVector<'_> {
    // SAFETY: the field pairs with a vector.
    unsafe { ForwardsUOffset::<RawVector<'_>>::follow(buf, position) }
}

/// A vector as its length and the position of its first element.
#[derive(Clone, Copy)]
pub(super) struct RawVector<'a> {
    buf: &'a [u8],
    /// The position of the length prefix.
    loc: usize,
    pub(super) len: usize,
}

impl<'a> RawVector<'a> {
    /// The bytes of the elements, `size` bytes each.
    pub(super) fn elements(&self, size: usize) -> &'a [u8] {
        let start = self.loc + SIZE_UOFFSET;
        &self.buf[start..start + self.len * size]
    }

    /// The elements as strings.
    pub(super) fn strings(&self) -> Vector<'a, ForwardsUOffset<&'a [u8]>> {
        // SAFETY: the vector pairs with strings.
        unsafe { Vector::new(self.buf, self.loc) }
    }

    /// The elements as tables.
    pub(super) fn tables(&self) -> Vector<'a, ForwardsUOffset<Table<'a>>> {
        // SAFETY: the vector pairs with tables.
        unsafe { Vector::new(self.buf, self.loc) }
    }
}

impl<'a> Follow<'a> for RawVector<'a> {
    type Inner = RawVector<'a>;

    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        let len = unsafe { UOffsetT::follow(buf, loc) } as usize;
        RawVector { buf, loc, len }
    }
}
