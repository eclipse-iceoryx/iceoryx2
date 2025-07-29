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
use std::fs::File;
use std::io::Write;

use crate::cli::DataRepresentation;

pub const HEX_START_RECORD_MARKER: &[u8] = b"### Recorded Data Start ###";

pub struct Record {
    pub timestamp: Duration,
    pub user_header: Vec<u8>,
    pub payload: Vec<u8>,
}

pub struct RecordCreator<'a> {
    file: &'a File,
    data_representation: DataRepresentation,
    time_stamp: Duration,
}

impl<'a> RecordCreator<'a> {
    pub fn new(file: &'a mut File) -> Self {
        Self {
            file,
            data_representation: DataRepresentation::default(),
            time_stamp: Duration::ZERO,
        }
    }

    pub fn data_representation(mut self, data_representation: DataRepresentation) -> Self {
        self.data_representation = data_representation;
        self
    }

    pub fn time_stamp(mut self, time: Duration) -> Self {
        self.time_stamp = time;
        self
    }

    pub fn write(mut self, user_header: &[u8], payload: &[u8]) -> Result<()> {
        match self.data_representation {
            DataRepresentation::Hex => {
                writeln!(self.file, "+{}", self.time_stamp.as_millis() as u64)?;
                writeln!(self.file, "{}", str::from_utf8(user_header)?)?;
                writeln!(self.file, "{}", str::from_utf8(payload)?)?;
            }
            DataRepresentation::Iox2Dump => {
                self.file
                    .write_all(&(self.time_stamp.as_millis() as u64).to_le_bytes())?;
                self.file
                    .write_all(&(user_header.len() as u64).to_le_bytes())?;
                self.file.write_all(user_header)?;
                self.file.write_all(&(payload.len() as u64).to_le_bytes())?;
                self.file.write_all(payload)?;
            }
        }

        Ok(())
    }
}
