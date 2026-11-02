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

extern crate alloc;

use crate::service::{
    builder::{ServiceCreateError, ServiceOpenError},
    marker::flatbuffer_type_name,
    static_config::message_type_details::TypeName as TypeNameString,
};
use alloc::sync::Arc;
use alloc::vec::Vec;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_flatbuffers::{
    FindSchemaFileError, SchemaError, TypeName, find_best_fitting_binary_schema_file,
    is_binary_flatbuffer_schema, root_type_name,
};
use iceoryx2_bb_posix::file::{AccessMode, FileBuilder, FileOpenError, FileReadError};
use iceoryx2_bb_system_types::{file_path::FilePath, path::Path};
use iceoryx2_log::fail;

#[derive(Debug)]
pub(crate) enum SchemaPathError {
    NoFlatbufferSchemaSearchPathConfigured,
    NoFittingSchemaFileFound,
    InsufficientPermissions,
    Interrupt,
    InternalError,
}

impl From<SchemaPathError> for ServiceCreateError {
    fn from(value: SchemaPathError) -> Self {
        match value {
            SchemaPathError::InternalError => ServiceCreateError::InternalFailure,
            SchemaPathError::InsufficientPermissions => ServiceCreateError::InsufficientPermissions,
            SchemaPathError::NoFittingSchemaFileFound
            | SchemaPathError::NoFlatbufferSchemaSearchPathConfigured => {
                ServiceCreateError::UnableToAcquireTypeDefinition
            }
            SchemaPathError::Interrupt => ServiceCreateError::Interrupt,
        }
    }
}

impl From<SchemaPathError> for ServiceOpenError {
    fn from(value: SchemaPathError) -> Self {
        match value {
            SchemaPathError::InternalError => ServiceOpenError::InternalFailure,
            SchemaPathError::InsufficientPermissions => ServiceOpenError::InsufficientPermissions,
            SchemaPathError::NoFittingSchemaFileFound
            | SchemaPathError::NoFlatbufferSchemaSearchPathConfigured => {
                ServiceOpenError::UnableToAcquireTypeDefinition
            }
            SchemaPathError::Interrupt => ServiceOpenError::Interrupt,
        }
    }
}

/// Why a flatbuffer type definition could not be resolved.
#[derive(Debug)]
pub(crate) enum ResolveTypeDefinitionError {
    /// The schema could not be found or read.
    Schema(SchemaPathError),
    /// The schema is not a binary flatbuffer schema.
    NotABinarySchema,
    /// The schema declares no `root_type`.
    NoRootType,
    /// The type name composed from the root type does not fit.
    TypeNameTooLong,
}

impl From<SchemaPathError> for ResolveTypeDefinitionError {
    fn from(error: SchemaPathError) -> Self {
        Self::Schema(error)
    }
}

/// A loaded schema and the type name of its root type.
#[derive(Debug, Clone)]
pub struct Schema {
    pub bytes: Arc<[u8]>,
    pub type_name: TypeNameString,
}

/// Where the schema of a flatbuffer payload comes from.
#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum SchemaSource {
    /// The best fitting file in the configured lookup path for the type name.
    Lookup(TypeName),
    /// A file given by an absolute path or by a path relative to the lookup
    /// path.
    File(FilePath),
    /// The schema bytes provided directly.
    Bytes(Arc<[u8]>),
    /// The schema the service already stores.
    Stored,
}

impl SchemaSource {
    /// Loads the schema and the type name of its root type.
    ///
    /// Fails when the schema cannot be read, is not a binary flatbuffer
    /// schema, declares no `root_type` or names one the type name cannot
    /// hold.
    pub(crate) fn load(
        &self,
        config: &crate::config::Config,
    ) -> Result<Option<Schema>, ResolveTypeDefinitionError> {
        let msg = "Unable to load type definition";
        let lookup_path = config.global.service.flatbuffer_schema_path.as_ref();

        let bytes: Arc<[u8]> = match self {
            Self::Stored => return Ok(None),
            Self::Bytes(bytes) => bytes.clone(),
            Self::File(path) => {
                if !is_binary_flatbuffer_schema(&path.file_name()) {
                    fail!(from self, with ResolveTypeDefinitionError::NotABinarySchema,
                        "{msg} since the type definition must be a binary flatbuffer schema and this file \"{path}\" is something else.");
                }
                let path = fail!(from self, when self.file_path(lookup_path, path),
                    to ResolveTypeDefinitionError,
                    "{msg} since the path of the schema file could not be determined.");
                let bytes = fail!(from self, when self.read_file(&path),
                    to ResolveTypeDefinitionError,
                    "{msg} since the schema file \"{path}\" could not be read.");
                bytes.into()
            }
            Self::Lookup(type_name) => {
                let path = fail!(from self, when self.lookup_file(lookup_path, type_name),
                    to ResolveTypeDefinitionError,
                    "{msg} since no schema file could be looked up.");
                let bytes = fail!(from self, when self.read_file(&path),
                    to ResolveTypeDefinitionError,
                    "{msg} since the schema file \"{path}\" could not be read.");
                bytes.into()
            }
        };

        let root_type = match root_type_name(&bytes) {
            Ok(root_type) => root_type,
            Err(SchemaError::NotABinarySchema) => {
                fail!(from self, with ResolveTypeDefinitionError::NotABinarySchema,
                    "{msg} since the schema is not a binary flatbuffer schema. Generate it with `flatc --schema --binary`.");
            }
            Err(SchemaError::NoRootType) => {
                fail!(from self, with ResolveTypeDefinitionError::NoRootType,
                    "{msg} since the schema declares no root_type. Add `root_type <Table>;` to the .fbs and regenerate the binary schema.");
            }
        };

        let Some(type_name) = flatbuffer_type_name(root_type) else {
            fail!(from self, with ResolveTypeDefinitionError::TypeNameTooLong,
                "{msg} since the root type \"{root_type}\" of the schema does not fit into the type name.");
        };

        Ok(Some(Schema { bytes, type_name }))
    }

    fn lookup_path(&self, lookup_path: Option<&Path>) -> Result<Path, SchemaPathError> {
        match lookup_path {
            Some(path) => Ok(*path),
            None => {
                fail!(from self, with SchemaPathError::NoFlatbufferSchemaSearchPathConfigured,
                    "Unable to acquire type definition schema path since the Config::global.service.flatbuffer-schema-path is required but not set. Either set a lookup path or provide an absolute path to the flatbuffer schema file in the builder.");
            }
        }
    }

    fn file_path(
        &self,
        lookup_path: Option<&Path>,
        file: &FilePath,
    ) -> Result<FilePath, SchemaPathError> {
        if file.path().is_absolute() {
            return Ok(*file);
        }
        let mut path = fail!(from self, when self.lookup_path(lookup_path),
            to SchemaPathError,
            "Unable to acquire type definition schema path since the relative path \"{file}\" cannot be resolved without a configured lookup path.");
        path.add_path_entry(&(*file).into()).unwrap();
        unsafe { Ok(FilePath::new_unchecked(path.as_bytes())) }
    }

    fn lookup_file(
        &self,
        lookup_path: Option<&Path>,
        type_name: &TypeName,
    ) -> Result<FilePath, SchemaPathError> {
        let msg = "Unable to acquire type definition schema path";
        let lookup_path = fail!(from self, when self.lookup_path(lookup_path),
            to SchemaPathError,
            "{msg} since no schema file can be looked up without a configured lookup path.");
        match find_best_fitting_binary_schema_file(type_name, &lookup_path) {
            Ok(Some(file)) => Ok(file),
            Ok(None) => {
                fail!(from self, with SchemaPathError::NoFittingSchemaFileFound,
                    "{msg} since no fitting flatbuffer schema file was found. Please provide the absolute path to a flatbuffer schema file in the builder.");
            }
            Err(FindSchemaFileError::InsufficientPermissions) => {
                fail!(from self, with SchemaPathError::InsufficientPermissions,
                    "{msg} since the lookup for a fitting flatbuffer schema file failed due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from self, with SchemaPathError::InternalError,
                    "{msg} since the lookup for a fitting flatbuffer schema file failed due to an internal error. [{e:?}]");
            }
        }
    }

    fn read_file(&self, schema_path: &FilePath) -> Result<Vec<u8>, SchemaPathError> {
        let msg = "Unable to read type definition schema file";
        let file = match FileBuilder::new(schema_path).open_existing(AccessMode::Read) {
            Ok(file) => file,
            Err(FileOpenError::FileDoesNotExist) => {
                fail!(from self, with SchemaPathError::NoFittingSchemaFileFound,
                    "{msg} since there is no type definition file at \"{schema_path}\".");
            }
            Err(FileOpenError::Interrupt) => {
                fail!(from self, with SchemaPathError::Interrupt,
                    "{msg} since the type definition open operation was interrupted by a signal.");
            }
            Err(FileOpenError::InsufficientPermissions) => {
                fail!(from self, with SchemaPathError::InsufficientPermissions,
                    "{msg} since the type definition file \"{schema_path}\" could not be opened due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from self, with SchemaPathError::InternalError,
                    "{msg} since the type definition file \"{schema_path}\" could not be opened due to an internal error. [{e:?}]");
            }
        };

        let mut buffer: Vec<u8> = Vec::new();
        match file.read_to_vector(&mut buffer) {
            Ok(_) => Ok(buffer),
            Err(FileReadError::Interrupt) => {
                fail!(from self, with SchemaPathError::Interrupt,
                    "{msg} since the type definition read operation was interrupted by a signal.");
            }
            Err(e) => {
                fail!(from self, with SchemaPathError::InternalError,
                    "{msg} due to an internal failure while reading the type definition. [{e:?}]");
            }
        }
    }
}
