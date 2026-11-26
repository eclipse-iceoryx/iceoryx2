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

use iceoryx2_integrations_ros2_link_adapter::cdr::writer;
use iceoryx2_link_backend::wire::region::{ForwardRegion, ResizeError};
use iceoryx2_log::{fail, origin};

use super::layout::{Layout, MemberKind};
use crate::model::{ByteOrder, ScalarElements, ScalarType, ScalarValue, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError<E> {
    /// A value cannot be taken from its source.
    Values(E),
    /// The region cannot hold the CDR.
    Region(ResizeError),
    /// A string or a sequence exceeds its length prefix.
    TooLong,
    /// The source of an array holds another number of elements than the
    /// array does.
    ArrayLength { expected: usize, actual: usize },
}

impl<E: core::fmt::Debug> core::fmt::Display for WriteError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "WriteError::{self:?}")
    }
}

impl<E: core::fmt::Debug> core::error::Error for WriteError<E> {}

impl<E> From<writer::CreateError> for WriteError<E> {
    fn from(error: writer::CreateError) -> Self {
        match error {
            writer::CreateError::Region(error) => WriteError::Region(error),
        }
    }
}

impl<E> From<writer::ScalarError> for WriteError<E> {
    fn from(error: writer::ScalarError) -> Self {
        match error {
            writer::ScalarError::Region(error) => WriteError::Region(error),
        }
    }
}

impl<E> From<writer::StringError> for WriteError<E> {
    fn from(error: writer::StringError) -> Self {
        match error {
            writer::StringError::Region(error) => WriteError::Region(error),
            writer::StringError::TooLong => WriteError::TooLong,
        }
    }
}

impl<E> From<writer::LengthError> for WriteError<E> {
    fn from(error: writer::LengthError) -> Self {
        match error {
            writer::LengthError::Region(error) => WriteError::Region(error),
            writer::LengthError::TooLong => WriteError::TooLong,
        }
    }
}

impl<E> From<writer::SequenceError> for WriteError<E> {
    fn from(error: writer::SequenceError) -> Self {
        match error {
            writer::SequenceError::Region(error) => WriteError::Region(error),
            writer::SequenceError::TooLong => WriteError::TooLong,
        }
    }
}

/// Writes `values` as the CDR of the message of `layout` into `region`.
///
/// Fails when a value cannot be taken from `values` or the region cannot
/// hold the CDR.
pub fn write<V: Values, R: ForwardRegion>(
    layout: &Layout,
    values: &V,
    region: R,
) -> Result<(), WriteError<V::Error>> {
    let origin = origin!("cdr::write");

    let mut writer = fail!(
        from origin,
        when writer::Writer::new(region),
        to WriteError<V::Error>,
        "Unable to start the CDR"
    );

    fail!(
        from origin,
        when write_message(layout, values, &mut writer),
        "Unable to write the values as CDR"
    );

    Ok(())
}

fn write_message<V: Values, R: ForwardRegion>(
    layout: &Layout,
    values: &V,
    writer: &mut writer::Writer<R>,
) -> Result<(), WriteError<V::Error>> {
    let origin = origin!("cdr::write_message");

    // Write the members in CDR order.
    for index in &layout.order {
        match layout.member(*index).kind {
            MemberKind::Scalar(_) => {
                let value = match values.scalar(*index) {
                    Ok(value) => value,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the scalar value {}", index.0
                        );
                    }
                };
                fail!(
                    from origin,
                    when write_scalar(writer, value),
                    to WriteError<V::Error>,
                    "Unable to write the scalar value {}", index.0
                );
            }
            MemberKind::String => {
                let bytes = match values.string(*index) {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the string value {}", index.0
                        );
                    }
                };
                fail!(
                    from origin,
                    when writer.write_string(bytes),
                    to WriteError<V::Error>,
                    "Unable to write the string value {}", index.0
                );
            }
            MemberKind::ScalarSequence(scalar_type) => {
                let elements = match values.scalar_elements(*index) {
                    Ok(elements) => elements,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the scalar sequence value {}", index.0
                        );
                    }
                };
                fail!(
                    from origin,
                    when write_scalar_sequence(writer, scalar_type, elements),
                    to WriteError<V::Error>,
                    "Unable to write the scalar sequence value {}", index.0
                );
            }
            MemberKind::ScalarArray {
                scalar_type,
                number_of_elements,
            } => {
                let elements = match values.scalar_elements(*index) {
                    Ok(elements) => elements,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the scalar array value {}", index.0
                        );
                    }
                };

                // A source without elements is written as zeros.
                let expected = number_of_elements as usize;
                let actual = elements.bytes.len() / scalar_type.number_of_bytes();
                if actual != 0 && actual != expected {
                    fail!(
                        from origin,
                        with WriteError::ArrayLength { expected, actual },
                        "The scalar array value {} holds {} elements and its source {}",
                        index.0, expected, actual
                    );
                }
                fail!(
                    from origin,
                    when write_scalar_array(writer, scalar_type, number_of_elements, elements),
                    to WriteError<V::Error>,
                    "Unable to write the scalar array value {}", index.0
                );
            }
            MemberKind::StringSequence => {
                let elements = match values.string_elements(*index) {
                    Ok(elements) => elements,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the string sequence value {}", index.0
                        );
                    }
                };
                fail!(
                    from origin,
                    when writer.write_length(elements.len()),
                    to WriteError<V::Error>,
                    "Unable to write the length of the string sequence value {}", index.0
                );
                fail!(
                    from origin,
                    when write_string_elements::<V, R>(writer, elements),
                    "Unable to write the string sequence value {}", index.0
                );
            }
            MemberKind::StringArray { number_of_elements } => {
                let elements = match values.string_elements(*index) {
                    Ok(elements) => elements,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the string array value {}", index.0
                        );
                    }
                };

                let expected = number_of_elements as usize;
                let actual = elements.len();
                if actual == 0 {
                    // The source holds no elements. Write an empty string for
                    // each.
                    for _ in 0..number_of_elements {
                        fail!(
                            from origin,
                            when writer.write_string(&[]),
                            to WriteError<V::Error>,
                            "Unable to write an empty string of the string array value {}", index.0
                        );
                    }
                } else if actual == expected {
                    fail!(
                        from origin,
                        when write_string_elements::<V, R>(writer, elements),
                        "Unable to write the string array value {}", index.0
                    );
                } else {
                    fail!(
                        from origin,
                        with WriteError::ArrayLength { expected, actual },
                        "The string array value {} holds {} elements and its source {}",
                        index.0, expected, actual
                    );
                }
            }
            MemberKind::MessageSequence(model) => {
                let elements = match values.message_elements(*index) {
                    Ok(elements) => elements,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the message sequence value {}", index.0
                        );
                    }
                };
                fail!(
                    from origin,
                    when writer.write_length(elements.len()),
                    to WriteError<V::Error>,
                    "Unable to write the length of the message sequence value {}", index.0
                );
                fail!(
                    from origin,
                    when write_message_elements::<V, R>(layout.repeated(model), elements, writer),
                    "Unable to write the message sequence value {}", index.0
                );
            }
            MemberKind::MessageArray {
                model,
                number_of_elements,
            } => {
                let elements = match values.message_elements(*index) {
                    Ok(elements) => elements,
                    Err(error) => {
                        fail!(
                            from origin,
                            with WriteError::Values(error),
                            "Unable to take the message array value {}", index.0
                        );
                    }
                };

                let expected = number_of_elements as usize;
                let actual = elements.len();
                if actual == 0 {
                    // The source holds no elements. Write a message of zeros
                    // for each.
                    for _ in 0..number_of_elements {
                        fail!(
                            from origin,
                            when write_zero_message(layout.repeated(model), writer),
                            "Unable to write a message of zeros of the message array value {}", index.0
                        );
                    }
                } else if actual == expected {
                    fail!(
                        from origin,
                        when write_message_elements::<V, R>(layout.repeated(model), elements, writer),
                        "Unable to write the message array value {}", index.0
                    );
                } else {
                    fail!(
                        from origin,
                        with WriteError::ArrayLength { expected, actual },
                        "The message array value {} holds {} elements and its source {}",
                        index.0, expected, actual
                    );
                }
            }
        }
    }

    Ok(())
}

fn write_scalar<R: ForwardRegion>(
    writer: &mut writer::Writer<R>,
    value: ScalarValue,
) -> Result<(), writer::ScalarError> {
    match value {
        ScalarValue::Bool(value) => writer.write_bool(value),
        ScalarValue::U8(value) => writer.write_u8(value),
        ScalarValue::I8(value) => writer.write_i8(value),
        ScalarValue::U16(value) => writer.write_u16(value),
        ScalarValue::I16(value) => writer.write_i16(value),
        ScalarValue::U32(value) => writer.write_u32(value),
        ScalarValue::I32(value) => writer.write_i32(value),
        ScalarValue::U64(value) => writer.write_u64(value),
        ScalarValue::I64(value) => writer.write_i64(value),
        ScalarValue::F32(value) => writer.write_f32(value),
        ScalarValue::F64(value) => writer.write_f64(value),
    }
}

fn write_scalar_sequence<R: ForwardRegion>(
    writer: &mut writer::Writer<R>,
    scalar_type: ScalarType,
    elements: ScalarElements<'_>,
) -> Result<(), writer::SequenceError> {
    let number_of_bytes = scalar_type.number_of_bytes();

    if elements.byte_order == ByteOrder::HOST || number_of_bytes == 1 {
        // The elements lie in the byte order of the CDR. Copy them in bulk.
        writer.write_sequence(elements.bytes, number_of_bytes)?;
    } else {
        // The elements lie in the other byte order. Write them one by one.
        writer.write_length(elements.bytes.len() / number_of_bytes)?;
        for bytes in elements.bytes.chunks_exact(number_of_bytes) {
            let value = ScalarValue::from_bytes(scalar_type, bytes, elements.byte_order);
            write_scalar(writer, value)?;
        }
    }

    Ok(())
}

fn write_scalar_array<R: ForwardRegion>(
    writer: &mut writer::Writer<R>,
    scalar_type: ScalarType,
    number_of_elements: u32,
    elements: ScalarElements<'_>,
) -> Result<(), writer::ScalarError> {
    let number_of_bytes = scalar_type.number_of_bytes();

    if elements.bytes.is_empty() {
        // The source holds no elements. Write a zero for each.
        let zero = ScalarValue::from_le_bytes(scalar_type, &[0; 8]);
        for _ in 0..number_of_elements {
            write_scalar(writer, zero)?;
        }
    } else if elements.byte_order == ByteOrder::HOST || number_of_bytes == 1 {
        // The elements lie in the byte order of the CDR. Copy them in bulk.
        writer.write_array(elements.bytes, number_of_bytes)?;
    } else {
        // The elements lie in the other byte order. Write them one by one.
        for bytes in elements.bytes.chunks_exact(number_of_bytes) {
            let value = ScalarValue::from_bytes(scalar_type, bytes, elements.byte_order);
            write_scalar(writer, value)?;
        }
    }

    Ok(())
}

fn write_string_elements<V: Values, R: ForwardRegion>(
    writer: &mut writer::Writer<R>,
    elements: V::StringElementIter<'_>,
) -> Result<(), WriteError<V::Error>> {
    let origin = origin!("cdr::write_string_elements");

    for element in elements {
        let bytes = match element {
            Ok(bytes) => bytes,
            Err(error) => {
                fail!(
                    from origin,
                    with WriteError::Values(error),
                    "Unable to take a string element"
                );
            }
        };
        fail!(
            from origin,
            when writer.write_string(bytes),
            to WriteError<V::Error>,
            "Unable to write a string element of {} bytes", bytes.len()
        );
    }

    Ok(())
}

fn write_message_elements<V: Values, R: ForwardRegion>(
    layout: &Layout,
    elements: V::MessageElementIter<'_>,
    writer: &mut writer::Writer<R>,
) -> Result<(), WriteError<V::Error>> {
    let origin = origin!("cdr::write_message_elements");

    for element in elements {
        let element = match element {
            Ok(element) => element,
            Err(error) => {
                fail!(
                    from origin,
                    with WriteError::Values(error),
                    "Unable to take a message element"
                );
            }
        };
        fail!(
            from origin,
            when write_message(layout, &element, writer),
            "Unable to write a message element"
        );
    }

    Ok(())
}

/// Writes a message of `layout` whose scalars are zero and whose strings and
/// sequences are empty.
fn write_zero_message<E, R: ForwardRegion>(
    layout: &Layout,
    writer: &mut writer::Writer<R>,
) -> Result<(), WriteError<E>> {
    for index in &layout.order {
        match layout.member(*index).kind {
            MemberKind::Scalar(scalar_type) => {
                write_scalar(writer, ScalarValue::from_le_bytes(scalar_type, &[0; 8]))?;
            }
            MemberKind::String => writer.write_string(&[])?,
            MemberKind::ScalarSequence(_)
            | MemberKind::StringSequence
            | MemberKind::MessageSequence(_) => writer.write_length(0)?,
            MemberKind::ScalarArray {
                scalar_type,
                number_of_elements,
            } => {
                for _ in 0..number_of_elements {
                    write_scalar(writer, ScalarValue::from_le_bytes(scalar_type, &[0; 8]))?;
                }
            }
            MemberKind::StringArray { number_of_elements } => {
                for _ in 0..number_of_elements {
                    writer.write_string(&[])?;
                }
            }
            MemberKind::MessageArray {
                model,
                number_of_elements,
            } => {
                for _ in 0..number_of_elements {
                    write_zero_message(layout.repeated(model), writer)?;
                }
            }
        }
    }

    Ok(())
}
