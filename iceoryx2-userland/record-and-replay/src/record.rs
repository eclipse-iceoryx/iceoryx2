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
use iceoryx2_bb_log::{debug, fail};
use iceoryx2_bb_posix::file::{File, FileWriteError};

use crate::{record_header::RecordHeader, replayer::ReplayerOpenError};

pub const HEX_START_RECORD_MARKER: &[u8] = b"### Recorded Data Start ###";

#[derive(Debug, Clone, Copy, Default)]
pub enum DataRepresentation {
    Iox2Dump,
    #[default]
    HumanReadable,
}

pub struct Record {
    pub timestamp: Duration,
    pub system_header: Vec<u8>,
    pub user_header: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct RecordReader {
    header: RecordHeader,
    data_representation: DataRepresentation,
}

impl RecordReader {
    pub(crate) fn new(header: &RecordHeader) -> Self {
        Self {
            header: header.clone(),
            data_representation: DataRepresentation::default(),
        }
    }

    pub(crate) fn data_representation(mut self, value: DataRepresentation) -> Self {
        self.data_representation = value;
        self
    }

    fn hex_string_to_raw_data(hex_string: &str) -> Result<Vec<u8>, ReplayerOpenError> {
        let mut hex_string = hex_string.to_string();
        hex_string.retain(|c| !c.is_whitespace());
        hex_string
            .split_ascii_whitespace()
            .map(|hex| {
                u8::from_str_radix(&hex, 16).map_err(|e| {
                    debug!(from "hex_string_to_raw_data()",
                        "Unable convert {hex} to hex-code ({e:?}).");
                    ReplayerOpenError::InvalidHexCode
                })
            })
            .collect::<Result<Vec<u8>, ReplayerOpenError>>()
    }

    fn verify_payload(&self, payload: &Vec<u8>, error_msg: &str) -> Result<(), ReplayerOpenError> {
        if (self.header.payload_type.variant == TypeVariant::FixedSize
            && payload.len() != self.header.payload_type.size)
            || (self.header.payload_type.variant == TypeVariant::Dynamic
                && payload.len() % self.header.payload_type.size != 0)
        {
            fail!(from self, with ReplayerOpenError::CorruptedPayloadRecord,
                                "{error_msg} since the payload record is corrupted (has wrong size).");
        }

        Ok(())
    }

    fn verify_user_header(
        &self,
        header: &Vec<u8>,
        error_msg: &str,
    ) -> Result<(), ReplayerOpenError> {
        if header.len() != self.header.payload_type.size {
            fail!(from self, with ReplayerOpenError::CorruptedUserHeaderRecord,
                                "{error_msg} since the user header record is corrupted (has wrong size).");
        }

        Ok(())
    }

    fn verify_system_header(
        &self,
        header: &Vec<u8>,
        error_msg: &str,
    ) -> Result<(), ReplayerOpenError> {
        if header.len() != self.header.payload_type.size {
            fail!(from self, with ReplayerOpenError::CorruptedSystemHeaderRecord,
                                "{error_msg} since the system header record is corrupted (has wrong size).");
        }

        Ok(())
    }

    pub(crate) fn read(self, file: &File) -> Result<Option<Record>, ReplayerOpenError> {
        let msg = "Unable to read next record";
        match self.data_representation {
            DataRepresentation::HumanReadable => {
                let mut timestamp = None;
                let mut system_header = None;
                let mut header = None;
                let mut line = String::new();
                while file.read_line_to_string(&mut line).unwrap() != 0 {
                    const READABLE_PREFIX_LEN: usize = 10;
                    if timestamp.is_none() {
                        timestamp = Some(Duration::from_millis(
                            line.as_str()[9..].parse::<u64>().unwrap(),
                        ));
                    } else if system_header.is_none() {
                        system_header = Some(Self::hex_string_to_raw_data(
                            &line.as_str()[READABLE_PREFIX_LEN..],
                        )?);
                    } else if header.is_none() {
                        header = Some(Self::hex_string_to_raw_data(
                            &line.as_str()[READABLE_PREFIX_LEN..],
                        )?);
                    } else {
                        let payload =
                            Self::hex_string_to_raw_data(&line.as_str()[READABLE_PREFIX_LEN..])?;
                        self.verify_payload(&payload, msg)?;
                        self.verify_user_header(header.as_ref().unwrap(), msg)?;
                        self.verify_system_header(system_header.as_ref().unwrap(), msg)?;

                        return Ok(Some(Record {
                            timestamp: timestamp.take().unwrap(),
                            system_header: system_header.take().unwrap(),
                            user_header: header.take().unwrap(),
                            payload: Self::hex_string_to_raw_data(&line.as_str()[9..])?,
                        }));
                    }
                }

                Ok(None)
            }
            DataRepresentation::Iox2Dump => {
                let read = |buffer: &mut [u8]| {
                    let len = fail!(from self, when file.read(buffer),
                        with ReplayerOpenError::FailedToReadFile,
                        "{msg} since the underlying file could not be read.");
                    if len != buffer.len() as u64 {
                        fail!(from self, with ReplayerOpenError::FailedToReadFile,
                            "{msg} since the record has a size of {len} and {} bytes are expected.",
                            buffer.len());
                    }

                    Ok(())
                };
                let mut buffer = [0u8; 8];
                read(&mut buffer)?;
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

                self.verify_payload(&payload, msg)?;
                self.verify_user_header(&user_header, msg)?;
                self.verify_user_header(&system_header, msg)?;
                Ok(Some(Record {
                    timestamp: Duration::from_millis(timestamp),
                    system_header: system_header,
                    user_header: user_header,
                    payload: payload,
                }))
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct RecordWriter<'a> {
    file: &'a mut File,
    data_representation: DataRepresentation,
    time_stamp: Duration,
}

impl<'a> RecordWriter<'a> {
    pub(crate) fn new(file: &'a mut File) -> Self {
        Self {
            file,
            data_representation: DataRepresentation::default(),
            time_stamp: Duration::ZERO,
        }
    }

    pub(crate) fn data_representation(mut self, data_representation: DataRepresentation) -> Self {
        self.data_representation = data_representation;
        self
    }

    pub(crate) fn time_stamp(mut self, time: Duration) -> Self {
        self.time_stamp = time;
        self
    }

    pub(crate) fn write(
        self,
        system_header: &[u8],
        user_header: &[u8],
        payload: &[u8],
    ) -> Result<(), FileWriteError> {
        let origin = format!("{self:?}");
        let mut write_to_file = |data| -> Result<(), FileWriteError> {
            fail!(from origin, when self.file.write(data),
                "Failed to write data record entry into file.");
            Ok(())
        };

        match self.data_representation {
            DataRepresentation::HumanReadable => {
                let time_stamp = format!("time:     {}\n", self.time_stamp.as_millis() as u64);
                write_to_file(time_stamp.as_bytes())?;
                write_to_file(b"sys head: ")?;
                write_to_file(system_header)?;
                write_to_file(b"\n")?;
                write_to_file(b"usr head: ")?;
                write_to_file(user_header)?;
                write_to_file(b"\n")?;
                write_to_file(b"payload:  ")?;
                write_to_file(payload)?;
                write_to_file(b"\n\n")?;
            }
            DataRepresentation::Iox2Dump => {
                let time_stamp = (self.time_stamp.as_millis() as u64).to_be_bytes();
                write_to_file(&time_stamp)?;
                let system_header_len = (system_header.len() as u64).to_le_bytes();
                write_to_file(&system_header_len)?;
                write_to_file(system_header)?;
                let user_header_len = (user_header.len() as u64).to_le_bytes();
                write_to_file(&user_header_len)?;
                write_to_file(user_header)?;
                let payload_len = (payload.len() as u64).to_le_bytes();
                write_to_file(&payload_len)?;
                write_to_file(payload)?;
            }
        }

        Ok(())
    }
}
