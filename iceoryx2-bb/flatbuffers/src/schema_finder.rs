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
use alloc::vec::Vec;
use iceoryx2_bb_container::string::String;
use iceoryx2_bb_elementary::code_style::{camel_to_snake_case, snake_to_upper_camel_case};
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_posix::directory::{Directory, DirectoryOpenError, DirectoryReadError};
use iceoryx2_bb_posix::file_type::FileType;
use iceoryx2_bb_posix::metadata::MetadataFromPathError;
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

/// Returns true if the provided [`FileName`] is a binary flatbuffer schema, otherwise false.
pub fn is_binary_flatbuffer_schema(name: &FileName) -> bool {
    let name = name.as_str();
    if let Some(pos) = name.rfind(".") {
        let suffix = &name[pos + 1..];
        suffix.eq_ignore_ascii_case("bfbs")
    } else {
        false
    }
}

/// The separator between the segments of a flatbuffer namespace.
const NAMESPACE_SEPARATOR: char = '.';

fn is_schema_for_type_name(file_name: &FileName, name: &str) -> bool {
    if !is_binary_flatbuffer_schema(file_name) {
        return false;
    }

    if let Some(pos) = file_name.as_str().rfind(".") {
        let name_without_extension = &file_name.as_str()[0..pos];
        name_without_extension.eq_ignore_ascii_case(name)
            || name_without_extension.eq_ignore_ascii_case(&camel_to_snake_case(name))
            || name_without_extension.eq_ignore_ascii_case(&snake_to_upper_camel_case(name))
    } else {
        false
    }
}

fn is_namespace(file_name: &FileName, namespace: &str) -> bool {
    let name = file_name.as_str();

    name.eq_ignore_ascii_case(namespace)
        || name.eq_ignore_ascii_case(&camel_to_snake_case(namespace))
        || name.eq_ignore_ascii_case(&snake_to_upper_camel_case(namespace))
}

/// Returns the best fitting schema file for a given [`TypeName`]. If no schema file could be found
/// [`None`] is returned.
///
/// The best fitting schema file in descending order:
///
/// 1. `namespace/name.bfbs`, one directory per namespace segment,
///    `my/namespace/name.bfbs` for `my.namespace.Name`
/// 2. `name.bfbs`
/// 3. `something/namespace/name.bfbs`
/// 4. `something/name.bfbs`
///
pub fn find_best_fitting_binary_schema_file(
    type_name: &TypeName,
    root_path: &Path,
) -> Result<Option<FilePath>, FindSchemaFileError> {
    let namespace: Vec<&str> = type_name
        .namespace
        .as_str()
        .split(NAMESPACE_SEPARATOR)
        .filter(|segment| !segment.is_empty())
        .collect();
    find(type_name.name.as_str(), &namespace, root_path)
}

/// Finds the schema of `name` in `root_path`, with the segments of its
/// `namespace` that no directory above spelled yet.
fn find(
    name: &str,
    namespace: &[&str],
    root_path: &Path,
) -> Result<Option<FilePath>, FindSchemaFileError> {
    let origin = format!("find_best_fitting_binary_schema_file({root_path:?})");
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
        | Err(DirectoryReadError::MetadataFromPathError(
            MetadataFromPathError::InsufficientPermissions,
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
    let mut namespace_directory = None;

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
                if is_schema_for_type_name(entry.name(), name) {
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
                if let Some((segment, within)) = namespace.split_first()
                    && is_namespace(entry.name(), segment)
                {
                    namespace_directory = Some((create_sub_path(root_path, entry.name())?, within));
                }
            }
            _ => continue,
        }
    }

    if let Some((directory, within)) = &namespace_directory
        && let Ok(Some(file)) = find(name, within, directory)
    {
        return Ok(Some(file));
    }

    if let Some(file) = schema_file {
        return Ok(Some(file));
    }

    for entry in &contents {
        if entry.metadata().file_type() == FileType::Directory
            && let Ok(Some(file)) =
                find(name, namespace, &create_sub_path(root_path, entry.name())?)
        {
            return Ok(Some(file));
        }
    }

    Ok(None)
}
