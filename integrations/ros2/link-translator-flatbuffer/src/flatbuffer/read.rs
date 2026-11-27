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

use flatbuffers::{
    Follow, ForwardsUOffset, UOffsetT, VOffsetT, Vector, VectorIter, read_scalar_at,
};
use iceoryx2_log::{fail, origin};

use super::layout::{
    FieldEntry, FieldKind, FieldPlacement, Layout, RootObject, TableIndex, TablePlacement,
};
use crate::model::{self, ByteOrder, ScalarElements, ScalarValue, ValueIndex, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    /// The flatbuffer is too short to hold its root offset.
    MissingRootOffset,
}

impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReadError::{self:?}")
    }
}

impl core::error::Error for ReadError {}

#[derive(Debug)]
pub struct StringElementIter<'a> {
    elements: VectorIter<'a, ForwardsUOffset<&'a [u8]>>,
}

impl<'a> Iterator for StringElementIter<'a> {
    type Item = Result<&'a [u8], Infallible>;

    fn next(&mut self) -> Option<Self::Item> {
        self.elements.next().map(Ok)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.elements.size_hint()
    }
}

impl ExactSizeIterator for StringElementIter<'_> {}

#[derive(Debug)]
pub struct MessageElements<'a> {
    /// The layout of one element.
    layout: &'a Layout,
    buffer: &'a [u8],
    /// Where the first element of the vector starts.
    first: Location,
    number_of_elements: usize,
}

impl model::MessageElements for MessageElements<'_> {
    type Error = Infallible;
    type Element<'a>
        = Reader<'a>
    where
        Self: 'a;

    fn number_of_elements(&self) -> usize {
        self.number_of_elements
    }

    fn element(&self, position: usize) -> Reader<'_> {
        let root = match self.layout.root {
            RootObject::Table => {
                let element = self.first.0 + position * size_of::<UOffsetT>();
                // SAFETY: the vector holds an offset at the element.
                let offset = unsafe { read_scalar_at::<UOffsetT>(self.buffer, element) };
                Location(element + offset as usize)
            }
            RootObject::Struct {
                number_of_bytes, ..
            } => Location(self.first.0 + position * usize::from(number_of_bytes)),
        };

        Reader {
            layout: self.layout,
            buffer: self.buffer,
            root,
        }
    }
}

/// A location within the bytes of a flatbuffer.
#[derive(Debug, Clone, Copy)]
struct Location(usize);

/// Reads the values of a layout from a flatbuffer.
#[derive(Debug)]
pub struct Reader<'a> {
    layout: &'a Layout,
    buffer: &'a [u8],
    /// Where the root table or the root struct of the layout starts.
    root: Location,
}

impl<'a> Reader<'a> {
    /// Creates a reader of the values described by `layout` in the provided
    /// `flatbuffer` bytes.
    ///
    /// Fails when the bytes are too short to hold a root offset.
    pub fn new(layout: &'a Layout, flatbuffer: &'a [u8]) -> Result<Self, ReadError> {
        let origin = origin!("flatbuffer::Reader::new");

        if flatbuffer.len() < size_of::<UOffsetT>() {
            fail!(
                from origin,
                with ReadError::MissingRootOffset,
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
            // SAFETY: the root of a layout with tables is a table.
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
            FieldPlacement::VectorElement { offset } => {
                Some(Location(self.root.0 + usize::from(offset.0)))
            }
        }
    }
}

impl Values for Reader<'_> {
    type Error = Infallible;
    type StringElementIter<'a>
        = StringElementIter<'a>
    where
        Self: 'a;
    type MessageElements<'a>
        = MessageElements<'a>
    where
        Self: 'a;

    /// The scalar value at `index`.
    ///
    /// It is the default of the field when the flatbuffer omits it.
    fn scalar(&self, index: ValueIndex) -> Result<ScalarValue, Infallible> {
        let field = self.layout.field(index);
        let FieldKind::Scalar {
            scalar_type,
            default,
        } = field.kind
        else {
            unreachable!("the value is a scalar in the model")
        };

        Ok(match self.locate_field(field) {
            Some(location) => ScalarValue::from_le_bytes(scalar_type, &self.buffer[location.0..]),
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
            (FieldKind::ScalarVector(scalar_type), Some(location)) => {
                let (first, number_of_elements) = locate_vector(buffer, location);
                &buffer[first.0..first.0 + number_of_elements * scalar_type.number_of_bytes()]
            }
            (
                FieldKind::ScalarArray {
                    scalar_type,
                    number_of_elements,
                },
                Some(location),
            ) => {
                let number_of_bytes = number_of_elements as usize * scalar_type.number_of_bytes();
                &buffer[location.0..location.0 + number_of_bytes]
            }
            (
                FieldKind::Scalar { .. }
                | FieldKind::String
                | FieldKind::StringVector
                | FieldKind::TableVector(_)
                | FieldKind::StructVector(_),
                Some(_),
            ) => {
                unreachable!("the value holds scalar elements in the model")
            }
        };

        Ok(ScalarElements {
            bytes,
            byte_order: ByteOrder::Little,
        })
    }

    /// The elements of the string sequence or string array value at `index`.
    ///
    /// None when the flatbuffer omits it.
    fn string_elements(&self, index: ValueIndex) -> Result<StringElementIter<'_>, Infallible> {
        let vector = match self.locate_field(self.layout.field(index)) {
            // SAFETY: the field holds a vector of strings.
            Some(location) => unsafe {
                ForwardsUOffset::<Vector<'_, ForwardsUOffset<&[u8]>>>::follow(
                    self.buffer,
                    location.0,
                )
            },
            None => Vector::default(),
        };

        Ok(StringElementIter {
            elements: vector.iter(),
        })
    }

    /// The elements of the message sequence or message array value at `index`.
    ///
    /// None when the flatbuffer omits it.
    fn message_elements(&self, index: ValueIndex) -> Result<MessageElements<'_>, Infallible> {
        let field = self.layout.field(index);
        let (FieldKind::TableVector(model) | FieldKind::StructVector(model)) = field.kind else {
            unreachable!("the value holds message elements in the model")
        };
        let (first, number_of_elements) = match self.locate_field(field) {
            Some(location) => locate_vector(self.buffer, location),
            None => (self.root, 0),
        };

        Ok(MessageElements {
            layout: self.layout.repeated(model),
            buffer: self.buffer,
            first,
            number_of_elements,
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
