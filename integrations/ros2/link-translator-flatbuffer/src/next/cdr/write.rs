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
use crate::next::model::{ScalarValue, Values};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError<E> {
    /// A value cannot be taken from its source.
    Values(E),
    /// The region cannot hold the CDR.
    Region(ResizeError),
    /// A string exceeds its length prefix.
    TooLong,
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
