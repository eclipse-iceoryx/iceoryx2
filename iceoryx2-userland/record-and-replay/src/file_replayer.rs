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

use anyhow::anyhow;
use anyhow::Result;
use core::mem::MaybeUninit;
use core::time::Duration;
use iceoryx2_cal::serialize::toml::Toml;
use iceoryx2_cal::serialize::Serialize;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;

use crate::record::DataRepresentation;
use crate::record::Record;
use crate::record::HEX_START_RECORD_MARKER;
use crate::record_file_header::RecordFileHeader;

fn hex_string_to_raw_data(hex_string: &str) -> Result<Vec<u8>> {
    let mut hex_string = hex_string.to_string();
    hex_string.retain(|c| !c.is_whitespace());
    hex_string
        .split_ascii_whitespace()
        .map(|hex| {
            u8::from_str_radix(&hex, 16)
                .map_err(|e| anyhow::anyhow!("Invalid hex input at position {}.", e))
        })
        .collect::<Result<Vec<u8>>>()
}

pub struct FileReplayer {
    file: BufReader<File>,
    data_representation: DataRepresentation,
    data_position: usize,
    header: RecordFileHeader,
    buffer: Vec<Record>,
}

impl FileReplayer {
    pub fn open(file_name: &str, data_representation: DataRepresentation) -> Result<Self> {
        let mut file = BufReader::new(File::open(file_name)?);
        let (header, data_position) = Self::read_file_header(&mut file, data_representation)?;
        Ok(Self {
            data_representation,
            data_position,
            buffer: vec![],
            header,
            file,
        })
    }

    pub fn fill_buffer(&mut self) -> Result<()> {
        match self.data_representation {
            DataRepresentation::Hex => {
                let mut timestamp = None;
                let mut header = None;
                for line in (&mut self.file)
                    .lines()
                    .enumerate()
                    .skip(self.data_position)
                {
                    if timestamp.is_none() {
                        timestamp =
                            Some(Duration::from_millis(line.1?.as_str()[1..].parse::<u64>()?));
                    } else if header.is_none() {
                        header = Some(hex_string_to_raw_data(&line.1?)?);
                    } else {
                        self.buffer.push(Record {
                            timestamp: timestamp.take().unwrap(),
                            user_header: header.take().unwrap(),
                            payload: hex_string_to_raw_data(&line.1?)?,
                        })
                    }
                }
            }
            DataRepresentation::Iox2Dump => {
                let mut buffer = [0u8; 8];
                let mut read_buffer = || -> Result<()> {
                    self.file.read_exact(&mut buffer)?;
                    let timestamp = u64::from_le_bytes(buffer);

                    self.file.read_exact(&mut buffer)?;
                    let header_len = u64::from_le_bytes(buffer);
                    let mut header = vec![0u8; header_len as usize];
                    self.file.read_exact(&mut header)?;

                    self.file.read_exact(&mut buffer)?;
                    let payload_len = u64::from_le_bytes(buffer);
                    let mut payload = vec![0u8; payload_len as usize];
                    self.file.read_exact(&mut payload)?;

                    self.buffer.push(Record {
                        timestamp: Duration::from_millis(timestamp),
                        user_header: header,
                        payload: payload,
                    });

                    Ok(())
                };

                while read_buffer().is_ok() {}
            }
        }

        Ok(())
    }

    pub fn header(&self) -> &RecordFileHeader {
        &self.header
    }

    pub fn buffer(&self) -> &Vec<Record> {
        &self.buffer
    }

    fn read_file_header(
        file: &mut BufReader<File>,
        data_representation: DataRepresentation,
    ) -> Result<(RecordFileHeader, usize)> {
        match data_representation {
            DataRepresentation::Hex => {
                let mut buffer: Vec<u8> = vec![];
                let mut data_position = 0;
                for (line_nr, line) in file.lines().enumerate() {
                    let line = line?;
                    if line.as_bytes() == HEX_START_RECORD_MARKER {
                        data_position = line_nr;
                        break;
                    }
                    buffer.extend_from_slice(line.as_bytes());
                }

                Ok((
                    Toml::deserialize::<RecordFileHeader>(buffer.as_slice())
                        .map_err(|e| anyhow!("Failed to deserialize RecordFileHeader ({e:?})."))?,
                    data_position,
                ))
            }
            DataRepresentation::Iox2Dump => {
                let mut header = MaybeUninit::<RecordFileHeader>::uninit();
                file.read_exact(unsafe {
                    core::slice::from_raw_parts_mut(
                        header.as_mut_ptr() as *mut u8,
                        core::mem::size_of::<RecordFileHeader>(),
                    )
                })?;

                Ok((
                    unsafe { header.assume_init() },
                    core::mem::size_of::<RecordFileHeader>(),
                ))
            }
        }
    }
}
