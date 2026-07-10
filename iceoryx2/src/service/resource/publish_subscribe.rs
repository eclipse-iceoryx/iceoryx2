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

use crate::service;
use crate::service::builder::ServiceOpenError;
use crate::service::resource::{RemoveStaleResourcesError, ServiceResource};
use crate::{node::SharedNode, service::builder::ServiceCreateError};
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_flatbuffers::{FindSchemaFileError, TypeName, find_best_fitting_schema_file};
use iceoryx2_bb_posix::file::FileOpenError;
use iceoryx2_bb_posix::file::{AccessMode, FileBuilder, FileReadError};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_cal::event::{NamedConceptBuilder, NamedConceptMgmt};
use iceoryx2_cal::named_concept::{
    NamedConceptConfiguration, NamedConceptPathHintRemoveError, NamedConceptRemoveError,
};
use iceoryx2_cal::static_storage::{
    StaticStorage, StaticStorageBuilder, StaticStorageCreateError, StaticStorageOpenError,
    StaticStorageReadError,
};
use iceoryx2_log::{fail, warn};

const PAYLOAD_TYPE_DEFINITION: FileName = unsafe { FileName::new_unchecked_const(b"payload") };

pub struct PublishSubscribeResourceConfig<ServiceType: service::Service> {
    pub(crate) use_type_definition: bool,
    pub(crate) schema_path: Option<FilePath>,
    pub(crate) type_name: TypeName,
    pub(crate) shared_node: SharedNode<ServiceType>,
}

#[derive(Debug)]
pub struct PublishSubscribeResources<ServiceType: service::Service> {
    type_definition: Option<ServiceType::StaticStorage>,
    path_hint: Option<Path>,
    has_ownership: AtomicBool,
    _service_type: PhantomData<ServiceType>,
}

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

impl<ServiceType: service::Service> Abandonable for PublishSubscribeResources<ServiceType> {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        this.has_ownership.store(false, Ordering::Relaxed);
    }
}

impl<ServiceType: service::Service> Drop for PublishSubscribeResources<ServiceType> {
    fn drop(&mut self) {
        if let Some(path_hint) = &self.path_hint {
            drop(self.type_definition.take());
            if self.has_ownership.load(Ordering::Relaxed)
                && let Err(e) =
                    <ServiceType::StaticStorage as NamedConceptMgmt>::remove_path_hint(path_hint)
            {
                warn!(from self,
                        "Failed to remove the resource directory: \"{path_hint}\". [{e:?}]");
            }
        }
    }
}

impl<ServiceType: service::Service> ServiceResource for PublishSubscribeResources<ServiceType> {
    type Config = PublishSubscribeResourceConfig<ServiceType>;

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
        if let Some(s) = &self.type_definition {
            s.acquire_ownership();
        }
    }

    fn create(
        static_config: &crate::service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, crate::service::builder::ServiceCreateError> {
        let origin = "PublishSubscribeResourceConfig::create()";
        let msg = "Unable to create publish subscribe resources";

        if resource_config.use_type_definition {
            let config = Self::type_definition_static_storage_config(
                resource_config.shared_node.config(),
                static_config,
            );

            let schema_file_content = Self::read_schema_file(origin, msg, resource_config)?;

            let static_storage = match
                <<ServiceType::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<ServiceType::StaticStorage>>::new(&PAYLOAD_TYPE_DEFINITION).config(&config).create(&schema_file_content) {
                    Ok(static_storage) => static_storage,
                    Err(StaticStorageCreateError::Interrupt) => {
                        fail!(from origin, with ServiceCreateError::Interrupt,
                            "{msg} since the static storage creation for the type definition was interrupted by a signal.");
                    }
                    Err(StaticStorageCreateError::InsufficientPermissions) => {
                        fail!(from origin, with ServiceCreateError::InsufficientPermissions,
                            "{msg} since the static storage for the type definition could not be created due to insufficient permissions.");
                    }
                    Err(e) => {
                        fail!(from origin, with ServiceCreateError::InternalFailure,
                            "{msg} since the static storage for the type definition could not be created due to an internal failure. [{e:?}]");
                    }
                };
            static_storage.release_ownership();

            Ok(Self {
                type_definition: Some(static_storage),
                path_hint: Some(*config.get_path_hint()),
                has_ownership: AtomicBool::new(false),
                _service_type: PhantomData,
            })
        } else {
            Ok(Self {
                type_definition: None,
                path_hint: None,
                has_ownership: AtomicBool::new(false),
                _service_type: PhantomData,
            })
        }
    }

    fn open(
        static_config: &crate::service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, crate::service::builder::ServiceOpenError> {
        let origin = "PublishSubscribeResource::open()";
        let msg = "Unable to open publish subscribe resources";

        if resource_config.use_type_definition {
            let config = Self::type_definition_static_storage_config(
                resource_config.shared_node.config(),
                static_config,
            );

            let required_schema_content = Self::read_schema_file(origin, msg, resource_config)?;

            let static_storage = match
                <<ServiceType::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<ServiceType::StaticStorage>>::new(&PAYLOAD_TYPE_DEFINITION).config(&config).open(Duration::ZERO) {
                    Ok(static_storage) => static_storage,
                    Err(StaticStorageOpenError::InsufficientPermissions) => {
                        fail!(from origin, with ServiceOpenError::InsufficientPermissions,
                            "{msg} since the type definition could not be opened.");
                    }
                    Err(StaticStorageOpenError::Interrupt) => {
                        fail!(from origin, with ServiceOpenError::Interrupt,
                            "{msg} since the operation was interrupted by a signal.");
                    }
                    Err(StaticStorageOpenError::InitializationNotYetFinalized) => {
                        fail!(from origin, with ServiceOpenError::HangsInCreation,
                            "{msg} since the type defintion file is not yet initialized.");
                    }
                    Err(StaticStorageOpenError::DoesNotExist) => {
                        fail!(from origin, with ServiceOpenError::ServiceInCorruptedState,
                            "{msg} since the type definition file does not exist but it should be available.");
                    }
                    Err(e) => {
                        fail!(from origin, with ServiceOpenError::InternalFailure,
                            "{msg} due to an internal failure while opening the type definition storage. [{e:?}]");
                    }
                };

            let mut existing_schema_content = vec![0u8; static_storage.len() as usize];
            match static_storage.read(&mut existing_schema_content) {
                Ok(()) => (),
                Err(StaticStorageReadError::Interrupt) => {
                    fail!(from origin, with ServiceOpenError::Interrupt,
                       "{msg} since the read operation was interrupted by a signal.");
                }
                Err(StaticStorageReadError::StaticStorageWasModified) => {
                    fail!(from origin, with ServiceOpenError::ServiceInCorruptedState,
                        "{msg} since the type definition was modified after the service was created.");
                }
                Err(e) => {
                    fail!(from origin, with ServiceOpenError::InternalFailure,
                        "{msg} due to an internal failure while reading the type definition. [{e:?}]");
                }
            }

            if existing_schema_content != required_schema_content {
                fail!(from origin, with ServiceOpenError::IncompatiblePayload,
                    "{msg} since the payload defined in the provided type definition is not equal to the type definition of the service.");
            }

            static_storage.release_ownership();

            Ok(Self {
                has_ownership: AtomicBool::new(false),
                type_definition: Some(static_storage),
                path_hint: Some(*config.get_path_hint()),
                _service_type: PhantomData,
            })
        } else {
            Ok(Self {
                has_ownership: AtomicBool::new(false),
                type_definition: None,
                path_hint: None,
                _service_type: PhantomData,
            })
        }
    }

    unsafe fn remove_stale_resources(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError> {
        let origin = "PublishSubscribeResources::remove_stale_resources()";
        let msg = "Unable to remove stale publish subscribe resources";
        let storage_config = Self::type_definition_static_storage_config(config, static_config);
        match unsafe {
            <ServiceType::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::remove_cfg(
            &PAYLOAD_TYPE_DEFINITION, &storage_config,
        )
        } {
            Ok(_) => (),
            Err(NamedConceptRemoveError::Interrupt) => {
                fail!(from origin, with RemoveStaleResourcesError::InterruptedBySignal,
                    "{msg} since it was interrupted by a signal.");
            }
            Err(NamedConceptRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(NamedConceptRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} due to an internal failure.");
            }
        }

        let dir = Self::service_resource_directory(config, static_config);
        match <ServiceType::StaticStorage as NamedConceptMgmt>::remove_path_hint(&dir) {
            Ok(()) => Ok(()),
            Err(NamedConceptPathHintRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} since the resource directory could not be removed due to insufficient permissions.");
            }
            Err(NamedConceptPathHintRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} since the resource directory could not be removed due to an internal failure.");
            }
        }
    }
}

impl<ServiceType: service::Service> PublishSubscribeResources<ServiceType> {
    pub fn type_definition(&self) -> Option<&ServiceType::StaticStorage> {
        self.type_definition.as_ref()
    }

    fn type_definition_static_storage_config(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> <ServiceType::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::Configuration
    {
        let dir = Self::service_resource_directory(config, static_config);
        (<<ServiceType::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::Configuration as Default>::default())
            .path_hint(&dir)
            .prefix(&config.global.prefix)
            .suffix(&config.global.service.type_definition_suffix)
    }

    fn read_schema_file(
        origin: &str,
        msg: &str,
        resource_config: &PublishSubscribeResourceConfig<ServiceType>,
    ) -> Result<Vec<u8>, SchemaPathError> {
        let schema_path = Self::schema_path(origin, msg, resource_config)?;

        let file = match FileBuilder::new(&schema_path).open_existing(AccessMode::Read) {
            Ok(file) => file,
            Err(FileOpenError::FileDoesNotExist) => {
                fail!(from origin, with SchemaPathError::NoFittingSchemaFileFound,
                    "{msg} since there is no type definition file at \"{schema_path}\".");
            }
            Err(FileOpenError::Interrupt) => {
                fail!(from origin, with SchemaPathError::Interrupt,
                    "{msg} since the type definition open operation was interrupted by a signal.");
            }
            Err(FileOpenError::InsufficientPermissions) => {
                fail!(from origin, with SchemaPathError::InsufficientPermissions,
                    "{msg} since the type definition file \"{schema_path}\" could not be opened due to insufficient permissions.");
            }
            Err(e) => {
                fail!(from origin, with SchemaPathError::InternalError,
                    "{msg} since the type definition file \"{schema_path}\" could not be opened due to an internal error. [{e:?}]");
            }
        };

        let mut buffer: Vec<u8> = Vec::new();
        match file.read_to_vector(&mut buffer) {
            Ok(_) => Ok(buffer),
            Err(FileReadError::Interrupt) => {
                fail!(from origin, with SchemaPathError::Interrupt,
                    "{msg} since the type definition read operation was interrupted by a signal.");
            }
            Err(e) => {
                fail!(from origin, with SchemaPathError::InternalError,
                    "{msg} due to an internal failure while reading the type definition. [{e:?}]");
            }
        }
    }

    fn schema_path(
        origin: &str,
        msg: &str,
        resource_config: &PublishSubscribeResourceConfig<ServiceType>,
    ) -> Result<FilePath, SchemaPathError> {
        let flatbuffer_schema_path = || -> Result<Path, SchemaPathError> {
            match resource_config
                .shared_node
                .config()
                .global
                .service
                .flatbuffer_schema_path
            {
                Some(p) => Ok(p),
                None => {
                    fail!(from origin, with SchemaPathError::NoFlatbufferSchemaSearchPathConfigured,
                        "{msg} since the Config::global.service.flatbuffer-schema-path is required but not set. Either set a lookup path or provide an absolute path to the flatbuffer schema file in the builder.");
                }
            }
        };

        match resource_config.schema_path {
            Some(file_path) if file_path.path().is_absolute() => Ok(file_path),
            Some(file_path) => {
                let mut path = flatbuffer_schema_path()?;
                path.add_path_entry(&file_path.into()).unwrap();
                unsafe { Ok(FilePath::new_unchecked(path.as_bytes())) }
            }
            None => {
                match find_best_fitting_schema_file(
                    &resource_config.type_name,
                    &flatbuffer_schema_path()?,
                ) {
                    Ok(Some(file)) => Ok(file),
                    Ok(None) => {
                        fail!(from origin, with SchemaPathError::NoFittingSchemaFileFound,
                            "{msg} since no fitting flatbuffer schema file was found. Please provide the absolute path to a flatbuffer schema file in the builder.");
                    }
                    Err(FindSchemaFileError::InsufficientPermissions) => {
                        fail!(from origin, with SchemaPathError::InsufficientPermissions,
                            "{msg} since the lookup for a fitting flatbuffer schema file failed due to insufficient permissions.");
                    }
                    Err(e) => {
                        fail!(from origin, with SchemaPathError::InternalError,
                            "{msg} since the lookup for a fitting flatbuffer schema file failed due to an internal error. [{e:?}]");
                    }
                }
            }
        }
    }
}
