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

use core::time::Duration;

use iceoryx2_bb_flatbuffers::{FindSchemaFileError, TypeName, find_best_fitting_schema_file};
use iceoryx2_bb_posix::file::{AccessMode, FileBuilder, FileOpenError, FileReadError};
use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath, path::Path};
use iceoryx2_cal::{
    event::{NamedConceptBuilder, NamedConceptMgmt, SemanticString},
    named_concept::{
        NamedConceptConfiguration, NamedConceptPathHintRemoveError, NamedConceptRemoveError,
    },
    static_storage::{
        StaticStorage, StaticStorageBuilder, StaticStorageCreateError, StaticStorageOpenError,
        StaticStorageReadError, StaticStorageView,
    },
};
use iceoryx2_log::{fail, fatal_panic};

use crate::service::{
    builder::{ServiceCreateError, ServiceOpenError},
    resource::RemoveStaleResourcesError,
};

enum SchemaPathError {
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

#[derive(Debug)]
pub struct TypeDefinitionStorage<S: crate::service::Service> {
    pub storage: S::StaticStorage,
    pub path_hint: Path,
}

#[derive(Debug)]
pub struct TypeDefinition {
    pub use_type_definition: bool,
    pub schema_path: Option<FilePath>,
    pub type_name: TypeName,
}

impl TypeDefinition {
    pub fn create_storage<S: crate::service::Service>(
        &self,
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<TypeDefinitionStorage<S>>, ServiceCreateError> {
        if !self.use_type_definition {
            return Ok(None);
        }

        let msg = "Unable to create type definition storage";
        let static_storage_config =
            Self::type_definition_static_storage_config::<S>(config, static_config);

        let schema_file_content = self.read_schema_file(config)?;

        let static_storage = match
            <<S::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<S::StaticStorage>>::new(name).config(&static_storage_config).create(&schema_file_content) {
                    Ok(static_storage) => static_storage,
                    Err(StaticStorageCreateError::Interrupt) => {
                        fail!(from self, with ServiceCreateError::Interrupt,
                            "{msg} since the static storage creation for the type definition was interrupted by a signal.");
                    }
                    Err(StaticStorageCreateError::InsufficientPermissions) => {
                        fail!(from self, with ServiceCreateError::InsufficientPermissions,
                            "{msg} since the static storage for the type definition could not be created due to insufficient permissions.");
                    }
                    Err(e) => {
                        fail!(from self, with ServiceCreateError::InternalFailure,
                            "{msg} since the static storage for the type definition could not be created due to an internal failure. [{e:?}]");
                    }
                };
        static_storage.release_ownership();

        Ok(Some(TypeDefinitionStorage {
            storage: static_storage,
            path_hint: *static_storage_config.get_path_hint(),
        }))
    }

    pub fn open_storage<S: crate::service::Service>(
        &self,
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<TypeDefinitionStorage<S>>, ServiceOpenError> {
        if !self.use_type_definition {
            return Ok(None);
        }

        let msg = "Unable to open type definition storage";
        let static_storage_config =
            Self::type_definition_static_storage_config::<S>(config, static_config);

        let required_schema_content = self.read_schema_file(config)?;

        let static_storage = match
                <<S::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<S::StaticStorage>>::new(name).config(&static_storage_config).open(Duration::ZERO) {
                    Ok(static_storage) => static_storage,
                    Err(StaticStorageOpenError::InsufficientPermissions) => {
                        fail!(from self, with ServiceOpenError::InsufficientPermissions,
                            "{msg} since the type definition could not be opened.");
                    }
                    Err(StaticStorageOpenError::Interrupt) => {
                        fail!(from self, with ServiceOpenError::Interrupt,
                            "{msg} since the operation was interrupted by a signal.");
                    }
                    Err(StaticStorageOpenError::InitializationNotYetFinalized) => {
                        fail!(from self, with ServiceOpenError::HangsInCreation,
                            "{msg} since the type definition file is not yet initialized.");
                    }
                    Err(StaticStorageOpenError::DoesNotExist) => {
                        fail!(from self, with ServiceOpenError::ServiceInCorruptedState,
                            "{msg} since the type definition file does not exist but it should be available.");
                    }
                    Err(e) => {
                        fail!(from self, with ServiceOpenError::InternalFailure,
                            "{msg} due to an internal failure while opening the type definition storage. [{e:?}]");
                    }
                };

        let mut existing_schema_content = vec![0u8; static_storage.len() as usize];
        match static_storage.read(&mut existing_schema_content) {
            Ok(()) => (),
            Err(StaticStorageReadError::Interrupt) => {
                fail!(from self, with ServiceOpenError::Interrupt,
                       "{msg} since the read operation was interrupted by a signal.");
            }
            Err(StaticStorageReadError::StaticStorageWasModified) => {
                fail!(from self, with ServiceOpenError::ServiceInCorruptedState,
                        "{msg} since the type definition was modified after the service was created.");
            }
            Err(e) => {
                fail!(from self, with ServiceOpenError::InternalFailure,
                        "{msg} due to an internal failure while reading the type definition. [{e:?}]");
            }
        }

        if existing_schema_content != required_schema_content {
            fail!(from self, with ServiceOpenError::IncompatiblePayload,
                    "{msg} since the payload defined in the provided type definition is not equal to the type definition of the service.");
        }

        static_storage.release_ownership();

        Ok(Some(TypeDefinitionStorage {
            storage: static_storage,
            path_hint: *static_storage_config.get_path_hint(),
        }))
    }

    pub fn remove_stale_storage<S: crate::service::Service>(
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError> {
        let origin = "TypeDefinition::remove_stale_storage()";
        let msg = "Unable to remove stale type definition storage";
        let storage_config =
            Self::type_definition_static_storage_config::<S>(config, static_config);
        match unsafe {
            <S::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::remove_cfg(
                name,
                &storage_config,
            )
        } {
            Ok(_) => (),
            Err(NamedConceptRemoveError::Interrupt) => {
                fail!(from origin, with RemoveStaleResourcesError::InterruptedBySignal,
                    "{msg} {name} since it was interrupted by a signal.");
            }
            Err(NamedConceptRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} {name} due to insufficient permissions.");
            }
            Err(NamedConceptRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} {name} due to an internal failure.");
            }
        }

        let dir = Self::service_resource_directory(config, static_config);
        match <S::StaticStorage as NamedConceptMgmt>::remove_path_hint(&dir) {
            Ok(()) => Ok(()),
            Err(NamedConceptPathHintRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} {name} since the resource directory could not be removed due to insufficient permissions.");
            }
            Err(NamedConceptPathHintRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} since the resource directory could not be removed due to an internal failure.");
            }
        }
    }

    fn type_definition_static_storage_config<S: crate::service::Service>(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> <S::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::Configuration {
        let dir = Self::service_resource_directory(config, static_config);
        (<<S::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::Configuration as Default>::default())
            .path_hint(&dir)
            .prefix(&config.global.prefix)
            .suffix(&config.global.service.type_definition_suffix)
    }

    fn service_resource_directory(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Path {
        let origin = "TypeDefinition::service_resource_directory()";
        let mut root = config.global.service_dir();
        let id = fatal_panic!(from origin,
               when Path::new(static_config.unique_service_id().value().to_string().as_bytes()),
               "This should never happen! The service id is always a valid path name.");
        fatal_panic!(from origin,
                when root.add_path_entry(&id),
                "This should never happen! The full service directory is too long. A shorter iceoryx2 root path might solve the issue.");
        root
    }

    fn read_schema_file(&self, config: &crate::config::Config) -> Result<Vec<u8>, SchemaPathError> {
        let msg = "Unable to read type definition schema file";
        let schema_path = self.schema_path(config)?;

        let file = match FileBuilder::new(&schema_path).open_existing(AccessMode::Read) {
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

    fn schema_path(&self, config: &crate::config::Config) -> Result<FilePath, SchemaPathError> {
        let msg = "Unable to acquire type definition schema path";
        let flatbuffer_schema_path = || -> Result<Path, SchemaPathError> {
            match config.global.service.flatbuffer_schema_path {
                Some(p) => Ok(p),
                None => {
                    fail!(from self, with SchemaPathError::NoFlatbufferSchemaSearchPathConfigured,
                        "{msg} since the Config::global.service.flatbuffer-schema-path is required but not set. Either set a lookup path or provide an absolute path to the flatbuffer schema file in the builder.");
                }
            }
        };

        match self.schema_path {
            Some(file_path) if file_path.path().is_absolute() => Ok(file_path),
            Some(file_path) => {
                let mut path = flatbuffer_schema_path()?;
                path.add_path_entry(&file_path.into()).unwrap();
                unsafe { Ok(FilePath::new_unchecked(path.as_bytes())) }
            }
            None => {
                match find_best_fitting_schema_file(&self.type_name, &flatbuffer_schema_path()?) {
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
        }
    }
}
