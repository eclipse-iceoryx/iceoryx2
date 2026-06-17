// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::directory::Directory;
use iceoryx2_bb_posix::file_type::FileType;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;

fn is_flatbuffer_schema(name: &FileName) -> bool {
    false
}

pub fn find_best_fitting_schema_file<T>(root_path: &Path) -> FilePath {
    let name = crate::type_name::<T>();

    let dir = Directory::new(root_path).unwrap();
    let contents = dir.contents().unwrap();

    for entry in &contents {
        match entry.metadata().file_type() {
            FileType::File => {
                entry.name();
            }
            FileType::Directory => (),
            _ => continue,
        }
    }

    todo!()
}
