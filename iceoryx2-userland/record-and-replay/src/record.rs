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
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file::{File, FileWriteError};

pub const HEX_START_RECORD_MARKER: &[u8] = b"### Recorded Data Start ###";

#[derive(Debug, Clone, Copy, Default)]
pub enum DataRepresentation {
    Iox2Dump,
    #[default]
    Hex,
}

pub struct Record {
    pub timestamp: Duration,
    pub user_header: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct RecordCreator<'a> {
    file: &'a mut File,
    data_representation: DataRepresentation,
    time_stamp: Duration,
}

impl<'a> RecordCreator<'a> {
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

    pub(crate) fn write(self, user_header: &[u8], payload: &[u8]) -> Result<(), FileWriteError> {
        let origin = format!("{self:?}");
        let mut write_to_file = |data| -> Result<(), FileWriteError> {
            fail!(from origin, when self.file.write(data),
                "Failed to write data record entry into file.");
            Ok(())
        };

        match self.data_representation {
            DataRepresentation::Hex => {
                let time_stamp = format!("+{}", self.time_stamp.as_millis() as u64);
                write_to_file(time_stamp.as_bytes())?;
                write_to_file(b"\n")?;
                write_to_file(user_header)?;
                write_to_file(b"\n")?;
                write_to_file(payload)?;
                write_to_file(b"\n")?;
            }
            DataRepresentation::Iox2Dump => {
                let time_stamp = (self.time_stamp.as_millis() as u64).to_be_bytes();
                write_to_file(&time_stamp)?;
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
