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

use core::time::Duration;
use iceoryx2::prelude::MessagingPattern;
use iceoryx2::service::static_config::message_type_details::TypeDetail;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::file::{CreationMode, FileWriteError};
use iceoryx2_bb_posix::file::{File, FileBuilder};
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_cal::serialize::toml::Toml;
use iceoryx2_cal::serialize::Serialize;

use crate::record::DataRepresentation;
use crate::record::RecordCreator;
use crate::record::HEX_START_RECORD_MARKER;
use crate::record_file_header::RecordFileHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileRecorderCreateError {
    FailedToCreateRecordFile,
    UnableToWriteFile,
    UnableToSerializeRecordFileHeader,
}

impl core::fmt::Display for FileRecorderCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileRecorderCreateError::{self:?}")
    }
}

impl core::error::Error for FileRecorderCreateError {}

#[derive(Debug)]
pub struct FileRecorderBuilder {
    payload_type: TypeDetail,
    header_type: TypeDetail,
    data_representation: DataRepresentation,
    messaging_pattern: MessagingPattern,
}

impl FileRecorderBuilder {
    pub fn new(payload_type: &TypeDetail, header_type: &TypeDetail) -> Self {
        Self {
            payload_type: payload_type.clone(),
            header_type: header_type.clone(),
            data_representation: DataRepresentation::default(),
            messaging_pattern: MessagingPattern::PublishSubscribe,
        }
    }

    pub fn data_representation(mut self, value: DataRepresentation) -> Self {
        self.data_representation = value;
        self
    }

    pub fn messaging_pattern(mut self, value: MessagingPattern) -> Self {
        self.messaging_pattern = value;
        self
    }

    pub fn create(self, file_name: &FilePath) -> Result<FileRecorder, FileRecorderCreateError> {
        let msg = format!("Unable to create file recorder for \"{}\"", file_name);
        let mut file = match FileBuilder::new(file_name)
            .has_ownership(false)
            .creation_mode(CreationMode::CreateExclusive)
            .create()
        {
            Ok(v) => v,
            Err(e) => {
                fail!(from self, with FileRecorderCreateError::FailedToCreateRecordFile,
                    "{msg} since the underlying file could not be created ({e:?}).");
            }
        };

        self.write_file_header(
            &mut file,
            RecordFileHeader {
                version: PackageVersion::get().to_u64(),
                payload_type: self.payload_type.clone(),
                header_type: self.header_type.clone(),
                messaging_pattern: self.messaging_pattern,
            },
            self.data_representation,
        )?;

        Ok(FileRecorder {
            file,
            data_representation: self.data_representation,
        })
    }

    fn write_file_header(
        &self,
        file: &mut File,
        file_header: RecordFileHeader,
        data_representation: DataRepresentation,
    ) -> Result<(), FileRecorderCreateError> {
        match data_representation {
            DataRepresentation::Hex => self.write_hex_file_header(file, file_header),
            DataRepresentation::Iox2Dump => self.write_iox2dump_file_header(file, file_header),
        }
    }

    fn write_iox2dump_file_header(
        &self,
        file: &mut File,
        file_header: RecordFileHeader,
    ) -> Result<(), FileRecorderCreateError> {
        let msg = format!(
            "Unable to write RecordFileHeader into iox2dump file \"{:?}\"",
            file.path()
        );
        let buffer = unsafe {
            core::slice::from_raw_parts(
                (&file_header as *const RecordFileHeader) as *const u8,
                core::mem::size_of::<RecordFileHeader>(),
            )
        };

        fail!(from self,
                when file.write(buffer),
                with FileRecorderCreateError::UnableToWriteFile,
                "{msg} since the file could not be written.");

        Ok(())
    }

    fn write_hex_file_header(
        &self,
        file: &mut File,
        file_header: RecordFileHeader,
    ) -> Result<(), FileRecorderCreateError> {
        let msg = format!(
            "Unable to write RecordFileHeader into hex file \"{:?}\"",
            file.path()
        );
        let serialized = fail!(from self,
                               when Toml::serialize(&file_header),
                               with FileRecorderCreateError::UnableToSerializeRecordFileHeader,
                               "{msg} since the RecordFileHeader could not be serialized.");

        let mut write_to_file = |data| -> Result<(), FileRecorderCreateError> {
            fail!(from self,
              when file.write(data),
              with FileRecorderCreateError::UnableToWriteFile,
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

pub struct FileRecorder {
    file: File,
    data_representation: DataRepresentation,
}

impl FileRecorder {
    pub fn write_payload(
        &mut self,
        user_header: &[u8],
        payload: &[u8],
        time_stamp: Duration,
    ) -> Result<(), FileWriteError> {
        RecordCreator::new(&mut self.file)
            .data_representation(self.data_representation)
            .time_stamp(time_stamp)
            .write(user_header, payload)
    }
}
