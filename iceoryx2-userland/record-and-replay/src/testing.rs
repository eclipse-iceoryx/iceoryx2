// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use crate::{
    record::RawRecord,
    recorder::{Recorder, RecorderWriteError},
};

/// Stores a [`RawRecord`] in the [`Recorder`] without verifying if the payload size matches
/// the type sizes defined in [`RecordHeader`](crate::record_header::RecordHeader).
///
/// In testing scenarios the safety requirements can be explicitly violated!
///
/// # Safety
///
///  * ensure that the data len in the [`RawRecord`] matches the corresponding type len in the
///    header
///  * The timestamp must be greater or equal to the timestamp of the previous entry.
pub unsafe fn recorder_write_unchecked(
    recorder: &mut Recorder,
    record: RawRecord,
) -> Result<(), RecorderWriteError> {
    recorder.write_unchecked(record)
}
