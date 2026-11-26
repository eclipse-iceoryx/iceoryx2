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

use flatbuffers::{Follow, ForwardsUOffset, UOffsetT, VOffsetT};
use iceoryx2_log::{fail, origin};

use super::layout::{FieldEntry, FieldKind, FieldPlacement, Layout};
use crate::next::model::{ScalarValue, ValueIndex, Values};

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

    /// Locates `field` in the flatbuffer.
    ///
    /// Returns None when the flatbuffer omits the field.
    fn locate_field(&self, field: &FieldEntry) -> Option<Location> {
        let FieldPlacement::Table(slot) = field.placement;
        // SAFETY: the root of a flatbuffer is a table.
        let table = unsafe { flatbuffers::Table::new(self.buffer, self.root.0) };

        locate_slot(table, slot.vtable_offset)
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
}

fn locate_slot(table: flatbuffers::Table<'_>, vtable_offset: VOffsetT) -> Option<Location> {
    match table.vtable().get(vtable_offset) {
        0 => None,
        offset => Some(Location(table.loc() + usize::from(offset))),
    }
}
