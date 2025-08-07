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

use anyhow::Result;
use core::time::Duration;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file::{File, FileReadLineState};

use crate::{
    hex_conversion::{bytes_to_hex_string, hex_string_to_bytes},
    record_header::RecordHeaderDetails,
    recorder::RecorderWriteError,
    replayer::ReplayerOpenError,
};

pub(crate) const HEX_START_RECORD_MARKER: &[u8] = b"### Recorded Data Start ###";

#[derive(Debug, Clone, Copy, Default)]
/// Defines the internal data representation in the recorded file.
pub enum DataRepresentation {
    /// Memory efficiently as a raw bytes
    Iox2Dump,
    #[default]
    /// Human Readable hex-codes for all payloads
    HumanReadable,
}

/// Represents a all the data required for a record captured by a receiver.
pub struct RawRecord<'a> {
    /// The time this data was captured.
    pub timestamp: Duration,
    /// The system header of the data.
    pub system_header: &'a [u8],
    /// The user header of the data.
    pub user_header: &'a [u8],
    /// The payload of the data.
    pub payload: &'a [u8],
}

#[derive(Debug)]
/// Represents a stored record.
pub struct Record {
    /// The time this data was captured.
    pub timestamp: Duration,
    /// The system header of the data.
    pub system_header: Vec<u8>,
    /// The user header of the data.
    pub user_header: Vec<u8>,
    /// The payload of the data.
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct RecordReader {
    header: RecordHeaderDetails,
    data_representation: DataRepresentation,
}

impl RecordReader {
    pub(crate) fn new(header: &RecordHeaderDetails) -> Self {
        Self {
            header: header.clone(),
            data_representation: DataRepresentation::default(),
        }
    }

    pub(crate) fn data_representation(mut self, value: DataRepresentation) -> Self {
        self.data_representation = value;
        self
    }

    fn verify_payload(&self, payload: &[u8], error_msg: &str) -> Result<(), ReplayerOpenError> {
        if (self.header.types.payload.variant() == TypeVariant::FixedSize
            && payload.len() != self.header.types.payload.size())
            || (self.header.types.payload.variant() == TypeVariant::Dynamic
                && payload.len() % self.header.types.payload.size() != 0)
        {
            fail!(from self, with ReplayerOpenError::CorruptedPayloadRecord,
                                "{error_msg} since the payload record is corrupted (has wrong size {}, expected {}).",
                                payload.len(), self.header.types.payload.size());
        }

        Ok(())
    }

    fn verify_user_header(&self, header: &[u8], error_msg: &str) -> Result<(), ReplayerOpenError> {
        if header.len() != self.header.types.user_header.size() {
            fail!(from self, with ReplayerOpenError::CorruptedUserHeaderRecord,
                                "{error_msg} since the system header record is corrupted (has wrong size {}, expected {}).",
                                header.len(), self.header.types.user_header.size());
        }

        Ok(())
    }

    fn verify_system_header(
        &self,
        header: &[u8],
        error_msg: &str,
    ) -> Result<(), ReplayerOpenError> {
        if header.len() != self.header.types.system_header.size() {
            fail!(from self, with ReplayerOpenError::CorruptedSystemHeaderRecord,
                                "{error_msg} since the system header record is corrupted (has wrong size {}, expected {}).",
                                header.len(), self.header.types.system_header.size());
        }

        Ok(())
    }

    fn verify_record(&self, record: &Record, error_msg: &str) -> Result<(), ReplayerOpenError> {
        self.verify_payload(&record.payload, error_msg)?;
        self.verify_user_header(&record.user_header, error_msg)?;
        self.verify_system_header(&record.system_header, error_msg)?;
        Ok(())
    }

    fn read_human_readable_from_file(
        &self,
        file: &File,
    ) -> Result<Option<Record>, ReplayerOpenError> {
        let msg = "Unable to read next record";
        let mut timestamp = None;
        let mut system_header = None;
        let mut header = None;
        loop {
            let mut line = String::new();
            match file.read_line_to_string(&mut line) {
                Ok(FileReadLineState::EndOfFile(_)) => break,
                Ok(FileReadLineState::LineLen(0)) => continue,
                Ok(FileReadLineState::LineLen(n)) => {
                    if n < READABLE_PREFIX_LEN {
                        fail!(from self, with ReplayerOpenError::CorruptedContent,
                            "{msg} since the content seems to be corrupted.");
                    }
                }
                Err(e) => {
                    fail!(from self, with ReplayerOpenError::FailedToReadFile,
                        "{msg} since the file could not be read ({e:?}).");
                }
            }

            const READABLE_PREFIX_LEN: usize = 10;
            if timestamp.is_none() {
                timestamp = Some(Duration::from_millis(fail!(from self,
                        when line.as_str()[READABLE_PREFIX_LEN..].parse::<u64>(),
                        with ReplayerOpenError::CorruptedTimeStamp,
                        "{msg} since the timestamp entry is corrupted.")));
            } else if system_header.is_none() {
                system_header = Some(hex_string_to_bytes(&line.as_str()[READABLE_PREFIX_LEN..])?);
            } else if header.is_none() {
                header = Some(hex_string_to_bytes(&line.as_str()[READABLE_PREFIX_LEN..])?);
            } else {
                let record = Record {
                    timestamp: timestamp.take().unwrap(),
                    system_header: system_header.take().unwrap(),
                    user_header: header.take().unwrap(),
                    payload: hex_string_to_bytes(&line.as_str()[READABLE_PREFIX_LEN..])?,
                };
                self.verify_record(&record, msg)?;

                return Ok(Some(record));
            }
        }

        Ok(None)
    }

    fn read_iox2dump_from_file(&self, file: &File) -> Result<Option<Record>, ReplayerOpenError> {
        let msg = "Unable to read next record";
        let read = |buffer: &mut [u8]| {
            let len = fail!(from self, when file.read(buffer),
                with ReplayerOpenError::FailedToReadFile,
                "{msg} since the underlying file could not be read.");

            if len == 0 {
                return Ok(false);
            }

            if len != buffer.len() as u64 {
                fail!(from self, with ReplayerOpenError::FailedToReadFile,
                    "{msg} since the record has a size of {len} and {} bytes are expected.",
                    buffer.len());
            }

            Ok(true)
        };
        let mut buffer = [0u8; 8];
        if !read(&mut buffer)? {
            return Ok(None);
        }
        let timestamp = u64::from_le_bytes(buffer);

        read(&mut buffer)?;
        let system_header_len = u64::from_le_bytes(buffer);
        let mut system_header = vec![0u8; system_header_len as usize];
        read(&mut system_header)?;

        read(&mut buffer)?;
        let user_header_len = u64::from_le_bytes(buffer);
        let mut user_header = vec![0u8; user_header_len as usize];
        read(&mut user_header)?;

        read(&mut buffer)?;
        let payload_len = u64::from_le_bytes(buffer);
        let mut payload = vec![0u8; payload_len as usize];
        read(&mut payload)?;

        let record = Record {
            timestamp: Duration::from_millis(timestamp),
            system_header,
            user_header,
            payload,
        };
        self.verify_record(&record, msg)?;

        Ok(Some(record))
    }

    pub(crate) fn read(self, file: &File) -> Result<Option<Record>, ReplayerOpenError> {
        match self.data_representation {
            DataRepresentation::HumanReadable => self.read_human_readable_from_file(file),
            DataRepresentation::Iox2Dump => self.read_iox2dump_from_file(file),
        }
    }
}

#[derive(Debug)]
pub(crate) struct RecordWriter<'a> {
    file: &'a mut File,
    data_representation: DataRepresentation,
}

impl<'a> RecordWriter<'a> {
    pub(crate) fn new(file: &'a mut File) -> Self {
        Self {
            file,
            data_representation: DataRepresentation::default(),
        }
    }

    pub(crate) fn data_representation(mut self, data_representation: DataRepresentation) -> Self {
        self.data_representation = data_representation;
        self
    }

    pub(crate) fn write(self, record: RawRecord) -> Result<(), RecorderWriteError> {
        let origin = format!("{self:?}");
        let mut write_to_file = |data| -> Result<(), RecorderWriteError> {
            match self.file.write(data) {
                Ok(_) => Ok(()),
                Err(e) => {
                    fail!(from origin,
                            with RecorderWriteError::FileWriteError(e),
                            "Failed to write data record entry into file ({e:?}).");
                }
            }
        };

        match self.data_representation {
            DataRepresentation::HumanReadable => {
                let time_stamp = format!("time:     {}\n", record.timestamp.as_millis() as u64);
                write_to_file(time_stamp.as_bytes())?;
                write_to_file(b"sys head: ")?;
                let hex_system_header = bytes_to_hex_string(record.system_header);
                write_to_file(hex_system_header.as_bytes())?;
                write_to_file(b"\n")?;
                write_to_file(b"usr head: ")?;
                let hex_user_header = bytes_to_hex_string(record.user_header);
                write_to_file(hex_user_header.as_bytes())?;
                write_to_file(b"\n")?;
                write_to_file(b"payload:  ")?;
                let hex_payload = bytes_to_hex_string(record.payload);
                write_to_file(hex_payload.as_bytes())?;
                write_to_file(b"\n\n")?;
            }
            DataRepresentation::Iox2Dump => {
                let time_stamp = (record.timestamp.as_millis() as u64).to_le_bytes();
                write_to_file(&time_stamp)?;
                let system_header_len = (record.system_header.len() as u64).to_le_bytes();
                write_to_file(&system_header_len)?;
                write_to_file(record.system_header)?;
                let user_header_len = (record.user_header.len() as u64).to_le_bytes();
                write_to_file(&user_header_len)?;
                write_to_file(record.user_header)?;
                let payload_len = (record.payload.len() as u64).to_le_bytes();
                write_to_file(&payload_len)?;
                write_to_file(record.payload)?;
            }
        }

        Ok(())
    }
}
