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

//! An iceoryx2 support library that helps to find schema files.

extern crate alloc;

use crate::TypeName;
use alloc::format;
use iceoryx2_bb_elementary::code_style::{camel_to_snake_case, snake_to_upper_camel_case};
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_posix::directory::{
    Directory, DirectoryOpenError, DirectoryReadError, DirectoryStatError,
};
use iceoryx2_bb_posix::file_type::FileType;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::fail;

enum_gen! { FindSchemaFileError
  entry:
    InsufficientPermissions,
    InvalidRootPath,
    FailedToOpenDirectory,
    FailedToReadDirectory,
    SchemaFilePathExceedsMaxSupportedPathLength
}

fn is_flatbuffer_schema(name: &FileName) -> bool {
    let name = name.as_str();
    if let Some(pos) = name.rfind(".") {
        let suffix = &name[pos + 1..];
        suffix.eq_ignore_ascii_case("fbs")
    } else {
        false
    }
}

fn is_schema_for_type_name(file_name: &FileName, type_name: &TypeName) -> bool {
    if !is_flatbuffer_schema(file_name) {
        return false;
    }

    if let Some(pos) = file_name.as_str().rfind(".") {
        let name_without_extension = &file_name.as_str()[0..pos];
        name_without_extension.eq_ignore_ascii_case(type_name.name)
            || name_without_extension.eq_ignore_ascii_case(&camel_to_snake_case(type_name.name))
            || name_without_extension
                .eq_ignore_ascii_case(&snake_to_upper_camel_case(type_name.name))
    } else {
        false
    }
}

fn is_namespace(file_name: &FileName, type_name: &TypeName) -> bool {
    let name = file_name.as_str();

    name.eq_ignore_ascii_case(type_name.namespace)
        || name.eq_ignore_ascii_case(&camel_to_snake_case(type_name.namespace))
        || name.eq_ignore_ascii_case(&snake_to_upper_camel_case(type_name.namespace))
}

/// Returns the best fitting schema file for a given [`TypeName`]. If no schema file could be found
/// [`None`] is returned.
///
/// The best fitting schema file in descending order:
///
/// 1. `namespace/name.fbs`
/// 2. `name.fbs`
/// 3. `something/namespace/name.fbs`
/// 4. `something/name.fbs`
///
pub fn find_best_fitting_schema_file(
    type_name: &TypeName,
    root_path: &Path,
) -> Result<Option<FilePath>, FindSchemaFileError> {
    let origin = format!("find_best_fitting_schema_file({root_path:?})");
    let msg = "Unable to find best fitting schema file";

    let dir = match Directory::new(root_path) {
        Ok(dir) => dir,
        Err(DirectoryOpenError::DoesNotExist) => return Ok(None),
        Err(DirectoryOpenError::InsufficientPermissions) => {
            fail!(from origin,
                  with FindSchemaFileError::InsufficientPermissions,
                  "{msg} since the directory could not be opened due to insufficient permissions.");
        }
        Err(DirectoryOpenError::NotADirectory) => {
            fail!(from origin,
                  with FindSchemaFileError::InvalidRootPath,
                  "{msg} since the provided path is a file, not a directory.");
        }
        Err(e) => {
            fail!(from origin,
                  with FindSchemaFileError::FailedToOpenDirectory,
                  "{msg} due to an internal failure while opening the directory. [{e:?}]"
            );
        }
    };

    let contents = match dir.contents() {
        Ok(contents) => contents,
        Err(DirectoryReadError::InsufficientPermissions)
        | Err(DirectoryReadError::DirectoryStatError(
            DirectoryStatError::InsufficientPermissions,
        )) => {
            fail!(from origin,
                with FindSchemaFileError::InsufficientPermissions,
                "{msg} since the directory content could not be listed due to insufficient permissions.");
        }
        Err(DirectoryReadError::DirectoryDoesNoLongerExist) => return Ok(None),
        Err(e) => {
            fail!(from origin,
                with FindSchemaFileError::FailedToReadDirectory,
                "{msg} since the directory contents could not be read due to an internal failure. [{e:?}]");
        }
    };

    let mut schema_file = None;
    let mut namespace_subdirectory = None;

    let create_sub_path = |path: &Path, name: &FileName| -> Result<Path, FindSchemaFileError> {
        let mut path = *path;
        match path.add_path_entry(&name.into()) {
            Ok(()) => Ok(path),
            Err(e) => {
                fail!(from origin, with FindSchemaFileError::SchemaFilePathExceedsMaxSupportedPathLength,
                    "{msg} since the subdirectory {path}/{name} exceeds the max supported path length. [{e:?}]");
            }
        }
    };

    for entry in &contents {
        match entry.metadata().file_type() {
            FileType::File => {
                if is_schema_for_type_name(entry.name(), type_name) {
                    match FilePath::from_path_and_file(root_path, entry.name()) {
                        Ok(file) => schema_file = Some(file),
                        Err(e) => {
                            fail!(from origin,
                                with FindSchemaFileError::SchemaFilePathExceedsMaxSupportedPathLength,
                                "{msg} since the schema file path \"{root_path}/{}\" exceeds the max supported path length. [{e:?}]", entry.name());
                        }
                    }
                }
            }
            FileType::Directory => {
                if is_namespace(entry.name(), type_name) {
                    namespace_subdirectory = Some(create_sub_path(root_path, entry.name())?);
                }
            }
            _ => continue,
        }
    }

    if let Some(dir) = &namespace_subdirectory
        && let Ok(Some(file)) = find_best_fitting_schema_file(type_name, dir)
    {
        return Ok(Some(file));
    }

    if let Some(file) = schema_file {
        return Ok(Some(file));
    }

    for entry in &contents {
        if entry.metadata().file_type() == FileType::Directory
            && let Ok(Some(file)) =
                find_best_fitting_schema_file(type_name, &create_sub_path(root_path, entry.name())?)
        {
            return Ok(Some(file));
        }
    }

    Ok(None)
}
