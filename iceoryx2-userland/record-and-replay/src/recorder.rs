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

//! ## Example
//!
//! The individual types used by [`ServiceTypes`](crate::recorder::ServiceTypes) can be acquired
//! from the [`StaticConfig`](iceoryx2::service::static_config::StaticConfig) of a
//! [`Service`](iceoryx2::service::Service). For publish susbcribe one can call for instance
//! [`publish_subscribe::StaticConfig::message_type_details()`](iceoryx2::service::static_config::publish_subscribe::StaticConfig::message_type_details())
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2_userland_record_and_replay::prelude::*;
//! use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
//! use iceoryx2::service::static_config::message_type_details::TypeVariant;
//! use core::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let service_types = ServiceTypes {
//!     payload: TypeDetail::new::<u64>(TypeVariant::FixedSize),
//!     user_header: TypeDetail::new::<()>(TypeVariant::FixedSize),
//!     system_header: TypeDetail::new::<u64>(TypeVariant::FixedSize),
//! };
//!
//! // create the file recorder
//! let mut recorder = RecorderBuilder::new(&service_types)
//!     .data_representation(DataRepresentation::HumanReadable)
//!     .messaging_pattern(MessagingPattern::PublishSubscribe)
//!     .create(&FilePath::new(b"recorded_data.iox2")?, &ServiceName::new("my-service")?)?;
//!
//! # iceoryx2_bb_posix::file::File::remove(&FilePath::new(b"recorded_data.iox2")?)?;
//!
//! // add some recorded data
//! recorder.write(RawRecord {
//!     timestamp: Duration::ZERO,
//!     system_header: &[0u8; 8],
//!     user_header: &[0u8; 0],
//!     payload: &[0u8; 8]
//! })?;
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2::prelude::{MessagingPattern, ServiceName};
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file::{CreationMode, FileCreationError, FileWriteError};
use iceoryx2_bb_posix::file::{File, FileBuilder};
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_cal::serialize::toml::Toml;
use iceoryx2_cal::serialize::Serialize;

use crate::record::RecordWriter;
use crate::record::HEX_START_RECORD_MARKER;
use crate::record::{DataRepresentation, RawRecord};
use crate::record_header::{
    RecordHeader, RecordHeaderDetails, FILE_FORMAT_HUMAN_READABLE_VERSION,
    FILE_FORMAT_IOX2_DUMP_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Errors that can occur when a new [`Recorder`] is created with
/// [`RecorderBuilder::create()`].
pub enum RecorderCreateError {
    /// The recorded file already exists.
    FileAlreadyExists,
    /// The record file could not be created.
    FailedToCreateRecordFile,
    /// The record file was created but cannot be written to.
    UnableToWriteFile,
    /// The record header could not be serialized.
    UnableToSerializeRecordHeader,
}

impl core::fmt::Display for RecorderCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RecorderCreateError::{self:?}")
    }
}

impl core::error::Error for RecorderCreateError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Errors that can occur when data is written with the [`Recorder`].
pub enum RecorderWriteError {
    /// The underlying file could not be written.
    FileWriteError(FileWriteError),
    /// The user wanted to write a system header that is not compatible with the [`ServiceTypes`]
    CorruptedSystemHeaderRecord,
    /// The user wanted to write a payload that is not compatible with the [`ServiceTypes`]
    CorruptedPayloadRecord,
    /// The user wanted to write a user header that is not compatible with the [`ServiceTypes`]
    CorruptedUserHeaderRecord,
    /// The record was older than the previously stored record. All records must have a
    /// monotonic timestamp - no time backward jumps.
    TimestampOlderThanPreviousRecord,
}

impl core::fmt::Display for RecorderWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RecorderWriteError::{self:?}")
    }
}

impl core::error::Error for RecorderWriteError {}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Clone)]
/// The types that are used by the iceoryx2 [`Service`](iceoryx2::service::Service)
pub struct ServiceTypes {
    /// Defines the type of the payload.
    pub payload: TypeDetail,
    /// Defines the type of the user header.
    pub user_header: TypeDetail,
    /// Defines the type of the iceoryx2 internal system header.
    pub system_header: TypeDetail,
}

#[derive(Debug)]
/// Builder to create a new [`Recorder`].
pub struct RecorderBuilder {
    types: ServiceTypes,
    data_representation: DataRepresentation,
    messaging_pattern: MessagingPattern,
}

impl RecorderBuilder {
    /// Creates a new [`RecorderBuilder`] for the given set of [`ServiceTypes`].
    pub fn new(types: &ServiceTypes) -> Self {
        Self {
            types: types.clone(),
            data_representation: DataRepresentation::default(),
            messaging_pattern: MessagingPattern::PublishSubscribe,
        }
    }

    /// Defines the data representation of the file content the [`Recorder`] will create.
    pub fn data_representation(mut self, value: DataRepresentation) -> Self {
        self.data_representation = value;
        self
    }

    /// Defines the messaging pattern of the recorded file.
    pub fn messaging_pattern(mut self, value: MessagingPattern) -> Self {
        self.messaging_pattern = value;
        self
    }

    /// Creates a new file with and writes the record header into it. On failure
    /// [`RecorderCreateError`] is returned describing the error.
    pub fn create(
        self,
        file_name: &FilePath,
        service_name: &ServiceName,
    ) -> Result<Recorder, RecorderCreateError> {
        let msg = format!("Unable to create file recorder for \"{file_name}\"");
        let mut file = match FileBuilder::new(file_name)
            .has_ownership(false)
            .creation_mode(CreationMode::CreateExclusive)
            .create()
        {
            Ok(v) => v,
            Err(FileCreationError::FileAlreadyExists) => {
                fail!(from self, with RecorderCreateError::FileAlreadyExists,
                    "{msg} since the file already exists.");
            }
            Err(e) => {
                fail!(from self, with RecorderCreateError::FailedToCreateRecordFile,
                    "{msg} since the underlying file could not be created ({e:?}).");
            }
        };

        let header = RecordHeader {
            service_name: service_name.clone(),
            iceoryx2_version: PackageVersion::get().into(),
            details: RecordHeaderDetails {
                file_format_version: match self.data_representation {
                    DataRepresentation::HumanReadable => FILE_FORMAT_HUMAN_READABLE_VERSION,
                    DataRepresentation::Iox2Dump => FILE_FORMAT_IOX2_DUMP_VERSION,
                },
                types: self.types.clone(),
                messaging_pattern: self.messaging_pattern,
            },
        };
        self.write_header(&mut file, &header, self.data_representation)?;

        Ok(Recorder {
            file,
            header,
            data_representation: self.data_representation,
            last_timestamp: 0,
        })
    }

    fn write_header(
        &self,
        file: &mut File,
        file_header: &RecordHeader,
        data_representation: DataRepresentation,
    ) -> Result<(), RecorderCreateError> {
        match data_representation {
            DataRepresentation::HumanReadable => self.write_hex_header(file, file_header),
            DataRepresentation::Iox2Dump => self.write_iox2dump_header(file, file_header),
        }
    }

    fn write_iox2dump_header(
        &self,
        file: &mut File,
        file_header: &RecordHeader,
    ) -> Result<(), RecorderCreateError> {
        let msg = format!(
            "Unable to write RecordHeader into iox2dump file \"{:?}\"",
            file.path()
        );
        let buffer = unsafe {
            core::slice::from_raw_parts(
                (file_header as *const RecordHeader) as *const u8,
                core::mem::size_of::<RecordHeader>(),
            )
        };

        fail!(from self,
                when file.write(buffer),
                with RecorderCreateError::UnableToWriteFile,
                "{msg} since the file could not be written.");

        Ok(())
    }

    fn write_hex_header(
        &self,
        file: &mut File,
        file_header: &RecordHeader,
    ) -> Result<(), RecorderCreateError> {
        let msg = format!(
            "Unable to write RecordFileHeader into hex file \"{:?}\"",
            file.path()
        );
        let serialized = fail!(from self,
                               when Toml::serialize(&file_header),
                               with RecorderCreateError::UnableToSerializeRecordHeader,
                               "{msg} since the RecordFileHeader could not be serialized.");

        let mut write_to_file = |data| -> Result<(), RecorderCreateError> {
            fail!(from self,
              when file.write(data),
              with RecorderCreateError::UnableToWriteFile,
              "{msg} since the file could not be written.");
            Ok(())
        };

        write_to_file(&serialized)?;
        write_to_file(b"\n\n")?;
        write_to_file(HEX_START_RECORD_MARKER)?;
        write_to_file(b"\n")?;

        Ok(())
    }
}

#[derive(Debug)]
/// Is created by [`RecorderBuilder`] and stores captured payload records into the underlying
/// file.
pub struct Recorder {
    file: File,
    data_representation: DataRepresentation,
    header: RecordHeader,
    last_timestamp: u64,
}

impl Recorder {
    /// Writes a captured record into the file.
    pub fn write(&mut self, record: RawRecord) -> Result<(), RecorderWriteError> {
        let msg = "Unable to write new record";

        if record.system_header.len() != self.header.details.types.system_header.size() {
            fail!(from self, with RecorderWriteError::CorruptedSystemHeaderRecord,
                "{msg} since the system header entry is corrupted. Expected a size of {} but provided a size of {}.",
                self.header.details.types.system_header.size(), record.system_header.len());
        }

        if record.user_header.len() != self.header.details.types.user_header.size() {
            fail!(from self, with RecorderWriteError::CorruptedUserHeaderRecord,
                "{msg} since the user header entry is corrupted. Expected a size of {} but provided a size of {}.",
                self.header.details.types.user_header.size(), record.user_header.len());
        }

        if self.header.details.types.payload.variant() == TypeVariant::FixedSize
            && record.payload.len() != self.header.details.types.payload.size()
        {
            fail!(from self, with RecorderWriteError::CorruptedPayloadRecord,
                "{msg} since the payload entry is corrupted. Expected a size of {} but provided a size of {}.",
                self.header.details.types.payload.size(), record.payload.len());
        }

        if self.header.details.types.payload.variant() == TypeVariant::Dynamic
            && record.payload.len() % self.header.details.types.payload.size() != 0
        {
            fail!(from self, with RecorderWriteError::CorruptedPayloadRecord,
                "{msg} since the payload entry is corrupted. Expected a size which is a multiple of {} but provided a size of {}.",
                self.header.details.types.payload.size(), record.payload.len());
        }

        let new_timestamp = record.timestamp.as_millis() as u64;
        if self.last_timestamp > new_timestamp {
            fail!(from self, with RecorderWriteError::TimestampOlderThanPreviousRecord,
                "{msg} since record timestamp is older than the previous record entry. Records are not allowed to jump back in time.");
        }
        self.last_timestamp = new_timestamp;

        self.write_unchecked(record)
    }

    pub(crate) fn write_unchecked(&mut self, record: RawRecord) -> Result<(), RecorderWriteError> {
        RecordWriter::new(&mut self.file)
            .data_representation(self.data_representation)
            .write(record)
    }

    /// Returns the [`RecordHeader`] of the underlying file.
    pub fn header(&self) -> &RecordHeader {
        &self.header
    }
}
