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

use iceoryx2_integrations_ros2_link_adapter::cdr::reader::{self, Endianness, Location};
use iceoryx2_log::{fail, origin};

use super::layout::{Layout, MemberKind};
use crate::model::MessageElements as _;
use crate::model::{
    self, ByteOrder, MessageView, ModelIndex, ScalarElements, ScalarType, ScalarValue, ValueIndex,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    /// The CDR ends before the message does.
    Truncated,
    /// The CDR names an encapsulation other than plain CDR.
    UnsupportedEncapsulation,
    InvalidBool,
    InvalidString,
}

impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReadError::{self:?}")
    }
}

impl core::error::Error for ReadError {}

impl From<reader::CreateError> for ReadError {
    fn from(error: reader::CreateError) -> Self {
        match error {
            reader::CreateError::Truncated => ReadError::Truncated,
            reader::CreateError::UnsupportedEncapsulation => ReadError::UnsupportedEncapsulation,
        }
    }
}

impl From<reader::ScalarError> for ReadError {
    fn from(error: reader::ScalarError) -> Self {
        match error {
            reader::ScalarError::Truncated => ReadError::Truncated,
        }
    }
}

impl From<reader::BoolError> for ReadError {
    fn from(error: reader::BoolError) -> Self {
        match error {
            reader::BoolError::Truncated => ReadError::Truncated,
            reader::BoolError::InvalidBool => ReadError::InvalidBool,
        }
    }
}

impl From<reader::StringError> for ReadError {
    fn from(error: reader::StringError) -> Self {
        match error {
            reader::StringError::Truncated => ReadError::Truncated,
            reader::StringError::InvalidString => ReadError::InvalidString,
        }
    }
}

#[derive(Debug)]
pub struct StringElementIter<'a> {
    reader: reader::Reader<'a>,
    number_of_remaining_elements: usize,
}

impl<'a> StringElementIter<'a> {
    /// The string elements of the member that starts at the provided `reader`.
    fn new(mut reader: reader::Reader<'a>, kind: MemberKind) -> Result<Self, ReadError> {
        let number_of_elements = match kind {
            MemberKind::StringSequence => reader.read_length()?,
            MemberKind::StringArray { number_of_elements } => number_of_elements as usize,
            MemberKind::Scalar(_)
            | MemberKind::String
            | MemberKind::ScalarSequence(_)
            | MemberKind::ScalarArray { .. }
            | MemberKind::MessageSequence(_)
            | MemberKind::MessageArray { .. } => {
                unreachable!("the value holds string elements in the model")
            }
        };

        Ok(Self {
            reader,
            number_of_remaining_elements: number_of_elements,
        })
    }
}

impl<'a> Iterator for StringElementIter<'a> {
    type Item = Result<&'a [u8], ReadError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.number_of_remaining_elements == 0 {
            return None;
        }
        self.number_of_remaining_elements -= 1;
        Some(self.reader.read_string().map_err(ReadError::from))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            self.number_of_remaining_elements,
            Some(self.number_of_remaining_elements),
        )
    }
}

impl ExactSizeIterator for StringElementIter<'_> {}

#[derive(Debug)]
pub struct LocatedMessage<'a> {
    elements: MessageElements<'a>,
}

impl LocatedMessage<'_> {
    pub fn view(&self) -> Reader<'_> {
        const POSITION_OF_MESSAGE: usize = 0;

        self.elements.element(POSITION_OF_MESSAGE)
    }
}

#[derive(Debug)]
pub struct MessageElements<'a> {
    /// The layout of one element.
    layout: &'a Layout,
    reader: reader::Reader<'a>,
    number_of_elements: usize,
    /// The location of every value of every element by value index
    /// in the [`Layout`].
    locations: Vec<Location>,
}

impl<'a> MessageElements<'a> {
    /// The `number_of_elements` messages of `layout` that start at the
    /// provided `reader`.
    fn new(
        mut reader: reader::Reader<'a>,
        layout: &'a Layout,
        number_of_elements: usize,
    ) -> Result<Self, ReadError> {
        let number_of_members = layout.members.len();

        // The number of elements comes from the CDR and is not trusted, so
        // the list grows as the messages are found.
        let mut locations = Vec::new();
        for position in 0..number_of_elements {
            let start = position * number_of_members;
            locations.resize(start + number_of_members, reader.location());
            walk_message(&mut reader, layout, &mut |index, location| {
                locations[start + usize::from(index.0)] = location;
            })?;
        }

        Ok(Self {
            layout,
            reader,
            number_of_elements,
            locations,
        })
    }
}

impl model::MessageElements for MessageElements<'_> {
    type Error = ReadError;
    type Element<'a>
        = Reader<'a>
    where
        Self: 'a;

    fn number_of_elements(&self) -> usize {
        self.number_of_elements
    }

    fn element(&self, position: usize) -> Reader<'_> {
        let number_of_members = self.layout.members.len();
        let start = position * number_of_members;

        Reader {
            layout: self.layout,
            reader: self.reader.at(self.reader.location()),
            locations: &self.locations[start..start + number_of_members],
        }
    }
}

/// Reads the values of a layout from a CDR reader.
#[derive(Debug)]
pub struct Reader<'a> {
    layout: &'a Layout,
    reader: reader::Reader<'a>,
    /// By value index in the [`Layout`].
    locations: &'a [Location],
}

impl Reader<'_> {
    fn location(&self, index: ValueIndex) -> Location {
        self.locations[usize::from(index.0)]
    }
}

impl MessageView for Reader<'_> {
    type Error = ReadError;
    type StringElementIter<'a>
        = StringElementIter<'a>
    where
        Self: 'a;
    type MessageElements<'a>
        = MessageElements<'a>
    where
        Self: 'a;

    fn scalar(&self, index: ValueIndex) -> Result<ScalarValue, ReadError> {
        read_scalar(
            &mut self.reader.at(self.location(index)),
            self.layout.member(index).kind,
        )
    }

    fn string(&self, index: ValueIndex) -> Result<&[u8], ReadError> {
        read_string(
            &mut self.reader.at(self.location(index)),
            self.layout.member(index).kind,
        )
    }

    fn scalar_elements(&self, index: ValueIndex) -> Result<ScalarElements<'_>, ReadError> {
        read_scalar_elements(
            &mut self.reader.at(self.location(index)),
            self.layout.member(index).kind,
        )
    }

    fn string_elements(&self, index: ValueIndex) -> Result<StringElementIter<'_>, ReadError> {
        read_string_elements(
            self.reader.at(self.location(index)),
            self.layout.member(index).kind,
        )
    }

    fn message_elements(&self, index: ValueIndex) -> Result<MessageElements<'_>, ReadError> {
        read_message_elements(self.reader.at(self.location(index)), self.layout, index)
    }
}

/// Locates the values described by `layout` in the provided `cdr` bytes.
///
/// Fails when the bytes do not hold the values of the layout.
pub fn locate<'a>(layout: &'a Layout, cdr: &'a [u8]) -> Result<LocatedMessage<'a>, ReadError> {
    const NUMBER_OF_MESSAGES: usize = 1;

    let origin = origin!("cdr::locate");

    let reader = fail!(
        from origin,
        when reader::Reader::new(cdr),
        to ReadError,
        "Unable to read the header of the CDR of {} bytes", cdr.len()
    );

    let elements = fail!(
        from origin,
        when MessageElements::new(reader, layout, NUMBER_OF_MESSAGES),
        "Unable to locate the values in the CDR of {} bytes", cdr.len()
    );

    Ok(LocatedMessage { elements })
}

/// Walks the members of the message of `layout` that starts at the provided
/// `reader`.
///
/// Calls `visit` with the location of each member and leaves the reader at
/// the first byte after the message. Fails when the bytes do not hold a
/// message of the layout.
fn walk_message(
    reader: &mut reader::Reader<'_>,
    layout: &Layout,
    visit: &mut dyn FnMut(ValueIndex, Location),
) -> Result<(), ReadError> {
    let origin = origin!("cdr::walk_message");

    // Go through the members in CDR order and report where each one starts.
    for index in &layout.order {
        visit(*index, reader.location());

        // Read the member to move the reader to the start of the next one.
        // The value is not stored by the walk.
        let read = match layout.member(*index).kind {
            kind @ MemberKind::Scalar(_) => read_scalar(reader, kind).map(|_| ()),
            kind @ MemberKind::String => read_string(reader, kind).map(|_| ()),
            kind @ (MemberKind::ScalarSequence(_) | MemberKind::ScalarArray { .. }) => {
                read_scalar_elements(reader, kind).map(|_| ())
            }
            kind @ (MemberKind::StringSequence | MemberKind::StringArray { .. }) => {
                skip_string_elements(reader, kind)
            }
            MemberKind::MessageSequence(_) | MemberKind::MessageArray { .. } => {
                skip_message_elements(reader, layout, *index)
            }
        };
        fail!(
            from origin,
            when read,
            "Unable to read the value {}", index.0
        );
    }

    Ok(())
}

fn read_scalar(
    reader: &mut reader::Reader<'_>,
    kind: MemberKind,
) -> Result<ScalarValue, ReadError> {
    let MemberKind::Scalar(scalar_type) = kind else {
        unreachable!("the value is a scalar in the model")
    };

    Ok(match scalar_type {
        ScalarType::Bool => ScalarValue::Bool(reader.read_bool()?),
        ScalarType::U8 => ScalarValue::U8(reader.read_u8()?),
        ScalarType::I8 => ScalarValue::I8(reader.read_i8()?),
        ScalarType::U16 => ScalarValue::U16(reader.read_u16()?),
        ScalarType::I16 => ScalarValue::I16(reader.read_i16()?),
        ScalarType::U32 => ScalarValue::U32(reader.read_u32()?),
        ScalarType::I32 => ScalarValue::I32(reader.read_i32()?),
        ScalarType::U64 => ScalarValue::U64(reader.read_u64()?),
        ScalarType::I64 => ScalarValue::I64(reader.read_i64()?),
        ScalarType::F32 => ScalarValue::F32(reader.read_f32()?),
        ScalarType::F64 => ScalarValue::F64(reader.read_f64()?),
    })
}

fn read_string<'a>(
    reader: &mut reader::Reader<'a>,
    kind: MemberKind,
) -> Result<&'a [u8], ReadError> {
    let MemberKind::String = kind else {
        unreachable!("the value is a string in the model")
    };

    Ok(reader.read_string()?)
}

fn read_scalar_elements<'a>(
    reader: &mut reader::Reader<'a>,
    kind: MemberKind,
) -> Result<ScalarElements<'a>, ReadError> {
    let bytes = match kind {
        MemberKind::ScalarSequence(scalar_type) => {
            reader.read_sequence(scalar_type.number_of_bytes())?
        }
        MemberKind::ScalarArray {
            scalar_type,
            number_of_elements,
        } => reader.read_array(number_of_elements as usize, scalar_type.number_of_bytes())?,
        MemberKind::Scalar(_)
        | MemberKind::String
        | MemberKind::StringSequence
        | MemberKind::StringArray { .. }
        | MemberKind::MessageSequence(_)
        | MemberKind::MessageArray { .. } => {
            unreachable!("the value holds scalar elements in the model")
        }
    };
    Ok(ScalarElements {
        bytes,
        byte_order: match reader.endianness() {
            Endianness::Little => ByteOrder::Little,
            Endianness::Big => ByteOrder::Big,
        },
    })
}

fn read_string_elements<'a>(
    reader: reader::Reader<'a>,
    kind: MemberKind,
) -> Result<StringElementIter<'a>, ReadError> {
    StringElementIter::new(reader, kind)
}

fn read_message_element_count(
    reader: &mut reader::Reader<'_>,
    layout: &Layout,
    index: ValueIndex,
) -> Result<(ModelIndex, usize), ReadError> {
    Ok(match layout.member(index).kind {
        MemberKind::MessageSequence(model) => (model, reader.read_length()?),
        MemberKind::MessageArray {
            model,
            number_of_elements,
        } => (model, number_of_elements as usize),
        MemberKind::Scalar(_)
        | MemberKind::String
        | MemberKind::ScalarSequence(_)
        | MemberKind::ScalarArray { .. }
        | MemberKind::StringSequence
        | MemberKind::StringArray { .. } => {
            unreachable!("the value holds message elements in the model")
        }
    })
}

fn read_message_elements<'a>(
    mut reader: reader::Reader<'a>,
    layout: &'a Layout,
    index: ValueIndex,
) -> Result<MessageElements<'a>, ReadError> {
    let (model, number_of_elements) = read_message_element_count(&mut reader, layout, index)?;
    MessageElements::new(reader, layout.repeated(model), number_of_elements)
}

fn skip_string_elements(
    reader: &mut reader::Reader<'_>,
    kind: MemberKind,
) -> Result<(), ReadError> {
    let mut elements = StringElementIter::new(reader.at(reader.location()), kind)?;
    for element in &mut elements {
        element?;
    }
    *reader = elements.reader;

    Ok(())
}

fn skip_message_elements(
    reader: &mut reader::Reader<'_>,
    layout: &Layout,
    index: ValueIndex,
) -> Result<(), ReadError> {
    let (model, number_of_elements) = read_message_element_count(reader, layout, index)?;
    for _ in 0..number_of_elements {
        walk_message(reader, layout.repeated(model), &mut |_, _| {})?;
    }

    Ok(())
}
