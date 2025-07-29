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
use std::fs::File;

use crate::cli::DataRepresentation;
use crate::record_file_header::RecordFileHeader;

pub struct FileReplayer {
    data_representation: DataRepresentation,
}

impl FileReplayer {
    pub fn open(
        file_name: &str,
        payload_type: &TypeDetail,
        header_type: &TypeDetail,
        data_representation: DataRepresentation,
        messaging_pattern: MessagingPattern,
    ) -> Result<Self> {
        let file = std::fs::OpenOptions::new().read(true).open(file_name)?;
        let expected_file_header = RecordFileHeader {
            version: PackageVersion::get().to_u64(),
            payload_type: payload_type.clone(),
            header_type: header_type.clone(),
            messaging_pattern,
        };

        let current_file_header = Self::read_file_header(&file, data_representation)?;
        if expected_file_header != current_file_header {
            return Err(anyhow!("Failed to open record file since the file header does not match. Expected ({expected_file_header:?}), file header in record file ({current_file_header:?})."));
        }

        Ok(Self {
            data_representation,
        })
    }

    fn read_file_header(
        file: &File,
        data_representation: DataRepresentation,
    ) -> Result<RecordFileHeader> {
        match data_representation {
            DataRepresentation::Hex => {
                todo!();
            }
            DataRepresentation::Iox2Dump => todo!(),
        }
    }
}
