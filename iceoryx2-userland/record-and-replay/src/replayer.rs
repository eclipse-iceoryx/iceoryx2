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

use core::mem::MaybeUninit;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::file::File;
use iceoryx2_bb_posix::file::FileBuilder;
use iceoryx2_bb_posix::file::FileReadLineState;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_cal::serialize::toml::Toml;
use iceoryx2_cal::serialize::Serialize;

use crate::record::DataRepresentation;
use crate::record::Record;
use crate::record::RecordReader;
use crate::record::HEX_START_RECORD_MARKER;
use crate::record_header::RecordHeader;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReplayerOpenError {
    InvalidHexCode,
    FailedToOpenFile,
    FailedToReadFile,
    ActualHeaderDoesNotMatchRequiredHeader,
    UnableToDeserializeRecordHeader,
    CorruptedSystemHeaderRecord,
    CorruptedPayloadRecord,
    CorruptedUserHeaderRecord,
    CorruptedContent,
}

#[derive(Debug)]
pub struct ReplayerOpener {
    file_path: FilePath,
    data_representation: DataRepresentation,
    required_header: Option<RecordHeader>,
}

impl ReplayerOpener {
    pub fn new(file_path: &FilePath) -> Self {
        Self {
            file_path: file_path.clone(),
            data_representation: DataRepresentation::default(),
            required_header: None,
        }
    }

    pub fn data_representation(mut self, value: DataRepresentation) -> Self {
        self.data_representation = value;
        self
    }

    pub fn require_header(mut self, header: &RecordHeader) -> Self {
        self.required_header = Some(header.clone());
        self
    }

    pub fn read_into_buffer(self) -> Result<(Vec<Record>, RecordHeader), ReplayerOpenError> {
        let mut replay = self.open()?;

        let mut buffer = vec![];
        while let Some(record) = replay.next_record()? {
            buffer.push(record);
        }

        Ok((buffer, replay.header().clone()))
    }

    pub fn open(self) -> Result<Replayer, ReplayerOpenError> {
        let msg = "Unable to read recorded data";
        let origin = format!("{self:?}");
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

        if let Some(required_header) = self.required_header {
            if required_header != actual_header {
                fail!(from origin, with ReplayerOpenError::ActualHeaderDoesNotMatchRequiredHeader,
                    "{msg} since the required header: {required_header:?} does not match the actual header {actual_header:?}.");
            }
        }

        Ok(Replayer {
            file,
            data_representation: self.data_representation,
            header: actual_header.clone(),
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
                        buffer_position += line_length as usize + 1;
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
pub struct Replayer {
    file: File,
    data_representation: DataRepresentation,
    header: RecordHeader,
}

impl Replayer {
    pub fn next_record(&mut self) -> Result<Option<Record>, ReplayerOpenError> {
        RecordReader::new(&self.header)
            .data_representation(self.data_representation)
            .read(&self.file)
    }

    pub fn header(&self) -> &RecordHeader {
        &self.header
    }
}
