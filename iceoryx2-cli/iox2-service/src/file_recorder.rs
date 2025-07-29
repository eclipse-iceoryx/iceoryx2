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
use iceoryx2::prelude::MessagingPattern;
use iceoryx2::service::static_config::message_type_details::TypeDetail;
use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_cal::serialize::toml::Toml;
use iceoryx2_cal::serialize::Serialize;
use std::fs::File;
use std::io::Write;

use crate::cli::DataRepresentation;
use crate::record_file_header::RecordFileHeader;

pub struct FileRecorder {
    file: File,
    data_representation: DataRepresentation,
}

impl FileRecorder {
    pub fn create(
        file_name: &str,
        payload_type: &TypeDetail,
        header_type: &TypeDetail,
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) -> Result<Self> {
        let mut file = std::fs::OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(file_name)?;

        Self::write_file_header(
            &mut file,
            RecordFileHeader {
                version: PackageVersion::get().to_u64(),
                payload_type: payload_type.clone(),
                header_type: header_type.clone(),
                messaging_pattern,
            },
            data_representation,
        )?;

        Ok(Self {
            file,
            data_representation,
        })
    }

    pub fn write_payload(&mut self, user_header: &[u8], payload: &[u8]) -> Result<()> {
        match self.data_representation {
            DataRepresentation::Hex => {
                writeln!(self.file, "{}", str::from_utf8(user_header)?)?;
                writeln!(self.file, "{}", str::from_utf8(payload)?)?;
            }
            DataRepresentation::Iox2Dump => {
                self.file
                    .write_all(&(user_header.len() as u64).to_le_bytes())?;
                self.file.write_all(user_header)?;
                self.file.write_all(&(payload.len() as u64).to_le_bytes())?;
                self.file.write_all(payload)?;
            }
        }

        Ok(())
    }

    fn write_file_header(
        file: &mut File,
        file_header: RecordFileHeader,
        data_representation: DataRepresentation,
    ) -> Result<()> {
        match data_representation {
            DataRepresentation::Hex => Self::write_hex_file_header(file, file_header),
            DataRepresentation::Iox2Dump => Self::write_iox2dump_file_header(file, file_header),
        }
    }

    fn write_iox2dump_file_header(file: &mut File, file_header: RecordFileHeader) -> Result<()> {
        let buffer = unsafe {
            core::slice::from_raw_parts(
                (&file_header as *const RecordFileHeader) as *const u8,
                core::mem::size_of::<RecordFileHeader>(),
            )
        };
        file.write(buffer)?;

        Ok(())
    }

    fn write_hex_file_header(file: &mut File, file_header: RecordFileHeader) -> Result<()> {
        let serialized = Toml::serialize(&file_header)
            .map_err(|e| anyhow!("Failed to serialize FileHeader ({e:?})."))?;

        file.write(&serialized)?;

        let data_start_comment = r#"
##############################
### start of recorded data ###
##############################

        "#;
        file.write(data_start_comment.as_bytes())?;

        Ok(())
    }
}
