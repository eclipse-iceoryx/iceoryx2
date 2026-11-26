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

use core::convert::Infallible;

use flatbuffers::{Follow, ForwardsUOffset, UOffsetT, VOffsetT, read_scalar_at};
use iceoryx2_log::{fail, origin};

use super::layout::{FieldEntry, FieldKind, FieldPlacement, Layout, TableIndex, TablePlacement};
use crate::next::model::{ByteOrder, ScalarElements, ScalarValue, ValueIndex, Values};

/// A flatbuffer that holds no root offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Truncated;

impl core::fmt::Display for Truncated {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Truncated")
    }
}

impl core::error::Error for Truncated {}

/// A location within the bytes of a flatbuffer.
#[derive(Debug, Clone, Copy)]
struct Location(usize);

/// Reads the values of a layout from a flatbuffer.
#[derive(Debug)]
pub struct Reader<'a> {
    layout: &'a Layout,
    buffer: &'a [u8],
    /// Where the root table starts.
    root: Location,
}

impl<'a> Reader<'a> {
    /// A reader of the values `layout` places in `flatbuffer`.
    ///
    /// Fails when the bytes hold no root offset.
    pub fn new(layout: &'a Layout, flatbuffer: &'a [u8]) -> Result<Self, Truncated> {
        let origin = origin!("flatbuffer::Reader::new");

        if flatbuffer.len() < size_of::<UOffsetT>() {
            fail!(
                from origin,
                with Truncated,
                "The flatbuffer of {} bytes holds no root offset", flatbuffer.len()
            );
        }
        // SAFETY: the flatbuffer is trusted to be well-formed.
        let root = unsafe { ForwardsUOffset::<flatbuffers::Table<'_>>::follow(flatbuffer, 0) };

        Ok(Self {
            layout,
            buffer: flatbuffer,
            root: Location(root.loc()),
        })
    }

    /// Locates the table at `index` in the flatbuffer.
    ///
    /// Returns None when the flatbuffer omits the table.
    fn locate_table(&self, index: TableIndex) -> Option<flatbuffers::Table<'a>> {
        let TablePlacement::Nested(slot) = self.layout.table(index).placement else {
            // SAFETY: the root of a flatbuffer is a table.
            return Some(unsafe { flatbuffers::Table::new(self.buffer, self.root.0) });
        };
        let holder = self.locate_table(slot.table)?;
        let location = locate_slot(holder, slot.vtable_offset)?;

        // SAFETY: the field holds a table.
        Some(unsafe { ForwardsUOffset::<flatbuffers::Table<'_>>::follow(holder.buf(), location.0) })
    }

    /// Locates `field` in the flatbuffer.
    ///
    /// Returns None when the flatbuffer omits the field.
    fn locate_field(&self, field: &FieldEntry) -> Option<Location> {
        match field.placement {
            FieldPlacement::Table(slot) => {
                locate_slot(self.locate_table(slot.table)?, slot.vtable_offset)
            }
            FieldPlacement::Struct { slot, offset } => {
                let location = locate_slot(self.locate_table(slot.table)?, slot.vtable_offset)?;
                Some(Location(location.0 + usize::from(offset.0)))
            }
        }
    }
}

impl Values for Reader<'_> {
    type Error = Infallible;

    /// The scalar value at `index`.
    ///
    /// It is the default of the field when the flatbuffer omits it.
    fn scalar(&self, index: ValueIndex) -> Result<ScalarValue, Infallible> {
        let field = self.layout.field(index);
        let FieldKind::Scalar { scalar, default } = field.kind else {
            unreachable!("the value is a scalar in the model")
        };

        Ok(match self.locate_field(field) {
            Some(location) => ScalarValue::from_le_bytes(scalar, &self.buffer[location.0..]),
            None => default,
        })
    }

    /// The string value at `index`.
    ///
    /// It is empty when the flatbuffer omits it.
    fn string(&self, index: ValueIndex) -> Result<&[u8], Infallible> {
        Ok(match self.locate_field(self.layout.field(index)) {
            // SAFETY: the field holds a string.
            Some(location) => unsafe { ForwardsUOffset::<&[u8]>::follow(self.buffer, location.0) },
            None => &[],
        })
    }

    /// The elements of the scalar sequence or scalar array value at `index`.
    ///
    /// None when the flatbuffer omits it.
    fn scalar_elements(&self, index: ValueIndex) -> Result<ScalarElements<'_>, Infallible> {
        let field = self.layout.field(index);
        let buffer = self.buffer;
        let bytes = match (field.kind, self.locate_field(field)) {
            (_, None) => &[],
            (FieldKind::ScalarVector(scalar), Some(location)) => {
                let (first, number_of_elements) = locate_vector(buffer, location);
                &buffer[first.0..first.0 + number_of_elements * scalar.number_of_bytes()]
            }
            (
                FieldKind::ScalarArray {
                    scalar,
                    number_of_elements,
                },
                Some(location),
            ) => {
                let number_of_bytes = number_of_elements as usize * scalar.number_of_bytes();
                &buffer[location.0..location.0 + number_of_bytes]
            }
            (FieldKind::Scalar { .. } | FieldKind::String, Some(_)) => {
                unreachable!("the value holds scalar elements in the model")
            }
        };

        Ok(ScalarElements {
            bytes,
            byte_order: ByteOrder::Little,
        })
    }
}

fn locate_slot(table: flatbuffers::Table<'_>, vtable_offset: VOffsetT) -> Option<Location> {
    match table.vtable().get(vtable_offset) {
        0 => None,
        offset => Some(Location(table.loc() + usize::from(offset))),
    }
}

/// The location of the first element and the number of elements of the
/// vector that the offset at `location` refers to.
fn locate_vector(buffer: &[u8], location: Location) -> (Location, usize) {
    // SAFETY: the field holds the offset to a vector, which starts with its
    // number of elements.
    let (vector, number_of_elements) = unsafe {
        let vector = location.0 + read_scalar_at::<UOffsetT>(buffer, location.0) as usize;
        (vector, read_scalar_at::<UOffsetT>(buffer, vector) as usize)
    };

    (Location(vector + size_of::<UOffsetT>()), number_of_elements)
}
