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

//! ## Examples
//!
//! ### Load Recorded Data Into Memory Buffer (Small Payload)
//!
//! The whole recorded file is loaded into memory. Useful, when the data is not that large.
//!
//! ```no_run
//! use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
//! use iceoryx2::service::static_config::message_type_details::TypeVariant;
//! use iceoryx2::prelude::*;
//! use iceoryx2_userland_record_and_replay::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//!
//! let replay = ReplayerOpener::new(&FilePath::new(b"recorded_data.iox2")?)
//!     .data_representation(DataRepresentation::HumanReadable)
//!     .open()?;
//!  let record_header = replay.header().clone();
//!  let buffer = replay.read_into_buffer().unwrap();
//!
//! println!("record header of service types {record_header:?}");
//!
//! for record in buffer {
//!     println!("payload: {:?}", record.payload);
//!     println!("user_header: {:?}", record.user_header);
//!     println!("system_header: {:?}", record.system_header);
//!     println!("timestamp: {:?}", record.timestamp);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Read Record One-By-One (Large Payload)
//!
//! The recorded file is opened and the records are read one-by-one.
//!
//! ```no_run
//! use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeNameString};
//! use iceoryx2::service::static_config::message_type_details::TypeVariant;
//! use iceoryx2::prelude::*;
//! use iceoryx2_userland_record_and_replay::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//!
//! let mut replayer = ReplayerOpener::new(&FilePath::new(b"recorded_data.iox2")?)
//!     .data_representation(DataRepresentation::HumanReadable)
//!     .open()?;
//!
//! println!("record header of service types {:?}", replayer.header());
//!
//! while let Some(record) = replayer.next_record()? {
//!     println!("payload: {:?}", record.payload);
//!     println!("user_header: {:?}", record.user_header);
//!     println!("system_header: {:?}", record.system_header);
//!     println!("timestamp: {:?}", record.timestamp);
//! }
//!
//! # Ok(())
//! # }
//! ```

use core::mem::MaybeUninit;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::file::File;
use iceoryx2_bb_posix::file::FileBuilder;
use iceoryx2_bb_posix::file::FileReadLineState;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_cal::serialize::toml::Toml;
use iceoryx2_cal::serialize::Serialize;

use crate::hex_conversion::HexToBytesConversionError;
use crate::record::DataRepresentation;
use crate::record::Record;
use crate::record::RecordReader;
use crate::record::HEX_START_RECORD_MARKER;
use crate::record_header::RecordHeader;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// Failures that can occur when a recorded file is opened with [`ReplayerOpener::open()`]
/// or [`Replayer::read_into_buffer()`].
pub enum ReplayerOpenError {
    /// The files payload contains invalid hex symbols
    InvalidHexCode,
    /// The recorded file could not be opened.
    FailedToOpenFile,
    /// The file could be opened but reading failed.
    FailedToReadFile,
    /// The record header could not be serialized.
    UnableToDeserializeRecordHeader,
    /// The system header record does not satisfy the type requirements from the [`RecordHeader`]
    CorruptedSystemHeaderRecord,
    /// The payload record does not satisfy the type requirements from the [`RecordHeader`]
    CorruptedPayloadRecord,
    /// The user header record does not satisfy the type requirements from the [`RecordHeader`]
    CorruptedUserHeaderRecord,
    /// The timestamp value is corrupted.
    CorruptedTimeStamp,
    /// The overall content of the file is corrupted.
    CorruptedContent,
    /// The file contains records that jump back and forth in time.
    CorruptedTimeline,
}

impl From<HexToBytesConversionError> for ReplayerOpenError {
    fn from(_value: HexToBytesConversionError) -> Self {
        Self::InvalidHexCode
    }
}

impl core::fmt::Display for ReplayerOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReplayerOpenError::{self:?}")
    }
}

impl core::error::Error for ReplayerOpenError {}

#[derive(Debug)]
/// Builder to open a recorded file. It returns either a in memory buffer and [`RecordHeader`]
/// with [`Replayer::read_into_buffer()`] (suggested for small payloads) or the
/// [`Replayer`] which can read the file entry by entry.
pub struct ReplayerOpener {
    file_path: FilePath,
    data_representation: DataRepresentation,
}

impl ReplayerOpener {
    /// Creates a new [`ReplayerOpener`]
    pub fn new(file_path: &FilePath) -> Self {
        Self {
            file_path: file_path.clone(),
            data_representation: DataRepresentation::default(),
        }
    }

    /// Defines the [`DataRepresentation`] of the file content.
    pub fn data_representation(mut self, value: DataRepresentation) -> Self {
        self.data_representation = value;
        self
    }

    /// Opens the recorded file and returns the [`Replayer`] which allows the user to
    /// read one entry at a time.
    pub fn open(self) -> Result<Replayer, ReplayerOpenError> {
        let msg = "Unable to read recorded data";
        let mut file = match FileBuilder::new(&self.file_path)
            .has_ownership(false)
            .open_existing(AccessMode::Read)
        {
            Ok(v) => v,
            Err(e) => {
                fail!(from self, with ReplayerOpenError::FailedToOpenFile,
                                "{msg} since the file could not be opened ({e:?}).");
            }
        };

        let actual_header = Self::read_header(&mut file, self.data_representation)?;

        Ok(Replayer {
            file,
            data_representation: self.data_representation,
            header: actual_header.clone(),
            last_timestamp: 0,
        })
    }

    fn read_header(
        file: &mut File,
        data_representation: DataRepresentation,
    ) -> Result<RecordHeader, ReplayerOpenError> {
        let msg = "Unable to read record file header";
        let origin = "read_header()";

        match data_representation {
            DataRepresentation::HumanReadable => {
                let mut buffer: Vec<u8> = vec![];
                let mut buffer_position = 0;

                loop {
                    let line_length = fail!(from origin, when file.read_line_to_vector(&mut buffer),
                            with ReplayerOpenError::FailedToReadFile,
                            "{msg} since the next line could not be read.");

                    if &buffer.as_slice()[buffer_position..] == HEX_START_RECORD_MARKER {
                        break;
                    }
                    buffer.push(b'\n');

                    if let FileReadLineState::LineLen(line_length) = line_length {
                        buffer_position += line_length + 1;
                    } else {
                        fail!(from origin,
                            with ReplayerOpenError::FailedToReadFile,
                            "{msg} since the file ends prematurely.");
                    }
                }

                let record_file_header = fail!(from origin,
                    when Toml::deserialize::<RecordHeader>(&buffer.as_slice()[0..buffer_position]),
                    with ReplayerOpenError::UnableToDeserializeRecordHeader,
                    "{msg} since the record header could not be deserialized.");

                Ok(record_file_header)
            }
            DataRepresentation::Iox2Dump => {
                let mut header = MaybeUninit::<RecordHeader>::uninit();
                let result = file.read(unsafe {
                    core::slice::from_raw_parts_mut(
                        header.as_mut_ptr() as *mut u8,
                        core::mem::size_of::<RecordHeader>(),
                    )
                });

                let read_bytes = fail!(from origin, when result,
                                    with ReplayerOpenError::FailedToReadFile,
                                    "{msg} since the record header could not be read.");

                if read_bytes != core::mem::size_of::<RecordHeader>() as u64 {
                    fail!(from origin, with ReplayerOpenError::UnableToDeserializeRecordHeader,
                        "{msg} since the record file entry is too short.");
                }

                Ok(unsafe { header.assume_init() })
            }
        }
    }
}

#[derive(Debug)]
/// Has read access to the recorded file and can extract one [`Record`] at a time.
pub struct Replayer {
    file: File,
    data_representation: DataRepresentation,
    header: RecordHeader,
    last_timestamp: u64,
}

impl Replayer {
    /// Reads the recorded file content into a buffer and returns it together with the
    /// contained [`RecordHeader`].
    pub fn read_into_buffer(mut self) -> Result<Vec<Record>, ReplayerOpenError> {
        let mut buffer = vec![];
        while let Some(record) = self.next_record()? {
            buffer.push(record);
        }

        Ok(buffer)
    }

    /// Returns the next contained [`Record`]. If it reached the end of the file it
    /// returns [`None`].
    pub fn next_record(&mut self) -> Result<Option<Record>, ReplayerOpenError> {
        if let Some(record) = RecordReader::new(&self.header.details)
            .data_representation(self.data_representation)
            .read(&self.file)?
        {
            let new_timestamp = record.timestamp.as_millis() as u64;
            if self.last_timestamp > new_timestamp {
                fail!(from self, with ReplayerOpenError::CorruptedTimeline,
                    "Unable to read next record since the next entries time stamp is older than the previous entries timestamp. The entries are not allowed to jump back and forth in time.");
            }

            self.last_timestamp = new_timestamp;

            return Ok(Some(record));
        }

        Ok(None)
    }

    /// Returns the header of the recorded file.
    pub fn header(&self) -> &RecordHeader {
        &self.header
    }
}
