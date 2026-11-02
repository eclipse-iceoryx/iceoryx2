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

use alloc::string::ToString;

use crate::service::{
    ServiceDetailsError,
    builder::{ServiceCreateError, ServiceOpenError},
    marker::is_flatbuffer_type_name,
    resource::{RemoveStaleResourcesError, schema_source::Schema},
    static_config::message_type_details::TypeDetail,
};
use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use iceoryx2_bb_container::string::String as _;
use iceoryx2_bb_system_types::{file_name::FileName, path::Path};
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

#[derive(Debug)]
pub struct TypeDefinitionStorage<S: crate::service::Service> {
    pub storage: S::StaticStorage,
    pub path_hint: Path,
}

impl<S: crate::service::Service> TypeDefinitionStorage<S> {
    /// Creates the type definition storage of a flatbuffer `payload` and
    /// writes `schema` into it. Any other payload has no storage and yields
    /// `None`.
    pub fn create(
        schema: Option<&Schema>,
        payload: &TypeDetail,
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<Self>, ServiceCreateError> {
        let origin = "TypeDefinitionStorage::create()";
        let msg = alloc::format!(
            "Unable to create type definition storage for \"{}\"",
            payload.type_name()
        );

        if !is_flatbuffer_type_name(payload.type_name().as_str()) {
            return Ok(None);
        }
        let Some(schema) = schema else {
            fail!(from origin, with ServiceCreateError::UnableToAcquireTypeDefinition,
                "{msg} since no schema was provided.");
        };

        let static_storage_config = Self::static_storage_config(config, static_config);

        let static_storage = match
            <<S::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<S::StaticStorage>>::new(name).config(&static_storage_config).create(&schema.bytes) {
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

        Ok(Some(Self {
            storage: static_storage,
            path_hint: *static_storage_config.get_path_hint(),
        }))
    }

    /// Copies the schema out of the type definition storage `name` of a
    /// service. The storage is not kept open.
    pub(crate) fn read_schema(
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Vec<u8>, ServiceDetailsError> {
        let origin = "TypeDefinitionStorage::read_schema()";
        let msg = "Unable to read type definition storage";

        let static_storage_config = Self::static_storage_config(config, static_config);

        let static_storage = match
                <<S::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<S::StaticStorage>>::new(name).config(&static_storage_config).open(Duration::ZERO) {
                    Ok(static_storage) => static_storage,
                    Err(StaticStorageOpenError::InsufficientPermissions) => {
                        fail!(from origin, with ServiceDetailsError::InsufficientPermissions,
                            "{msg} since the type definition could not be opened due to insufficient permissions.");
                    }
                    Err(StaticStorageOpenError::Interrupt) => {
                        fail!(from origin, with ServiceDetailsError::Interrupt,
                            "{msg} since the operation was interrupted by a signal.");
                    }
                    Err(StaticStorageOpenError::InitializationNotYetFinalized)
                    | Err(StaticStorageOpenError::DoesNotExist) => {
                        fail!(from origin, with ServiceDetailsError::ServiceInInconsistentState,
                            "{msg} since the type definition file is not available.");
                    }
                    Err(e) => {
                        fail!(from origin, with ServiceDetailsError::InternalError,
                            "{msg} due to an internal failure while opening the type definition storage. [{e:?}]");
                    }
                };
        static_storage.release_ownership();

        let mut bytes = vec![0u8; static_storage.len() as usize];
        match static_storage.read(&mut bytes) {
            Ok(()) => Ok(bytes),
            Err(StaticStorageReadError::Interrupt) => {
                fail!(from origin, with ServiceDetailsError::Interrupt,
                       "{msg} since the read operation was interrupted by a signal.");
            }
            Err(StaticStorageReadError::StaticStorageWasModified) => {
                fail!(from origin, with ServiceDetailsError::ServiceInInconsistentState,
                        "{msg} since the type definition was modified after the service was created.");
            }
            Err(e) => {
                fail!(from origin, with ServiceDetailsError::InternalError,
                        "{msg} due to an internal failure while reading the type definition. [{e:?}]");
            }
        }
    }

    /// Opens the type definition storage of a flatbuffer `payload` and checks
    /// that it holds `schema` when one is given. Any other payload has no
    /// storage and yields `None`.
    pub fn open(
        schema: Option<&Schema>,
        payload: &TypeDetail,
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<Self>, ServiceOpenError> {
        if !is_flatbuffer_type_name(payload.type_name().as_str()) {
            return Ok(None);
        }
        let Some(schema) = schema else {
            return Self::open_unverified(name, config, static_config);
        };

        let origin = "TypeDefinitionStorage::open()";
        let msg = alloc::format!(
            "Unable to open type definition storage for \"{}\"",
            payload.type_name()
        );
        let static_storage_config = Self::static_storage_config(config, static_config);

        let static_storage = match
                <<S::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<S::StaticStorage>>::new(name).config(&static_storage_config).open(Duration::ZERO) {
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
                            "{msg} since the type definition file is not yet initialized.");
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

        if existing_schema_content[..] != schema.bytes[..] {
            fail!(from origin, with ServiceOpenError::IncompatiblePayload,
                    "{msg} since the payload defined in the provided type definition is not equal to the type definition of the service.");
        }

        static_storage.release_ownership();

        Ok(Some(Self {
            storage: static_storage,
            path_hint: *static_storage_config.get_path_hint(),
        }))
    }

    pub fn remove_stale(
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError> {
        let origin = "TypeDefinitionStorage::remove_stale()";
        let msg = "Unable to remove stale type definition storage";
        let storage_config = Self::static_storage_config(config, static_config);
        match unsafe {
            <S::StaticStorage as iceoryx2_cal::named_concept::NamedConceptMgmt>::remove_cfg(
                name,
                &storage_config,
            )
        } {
            Ok(_) => Ok(()),
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
    }

    pub fn remove_resource_directory(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<(), RemoveStaleResourcesError> {
        let origin = "TypeDefinitionStorage::remove_resource_directory()";
        let msg = "Unable to remove resource directory";

        let dir = Self::service_resource_directory(config, static_config);
        match <S::StaticStorage as NamedConceptMgmt>::remove_path_hint(&dir) {
            Ok(()) => Ok(()),
            Err(NamedConceptPathHintRemoveError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStaleResourcesError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Err(NamedConceptPathHintRemoveError::InternalError) => {
                fail!(from origin, with RemoveStaleResourcesError::InternalFailure,
                    "{msg} due to an internal failure.");
            }
        }
    }

    fn open_unverified(
        name: &FileName,
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> Result<Option<Self>, ServiceOpenError> {
        let origin = "TypeDefinitionStorage::open_unverified()";
        let msg = "Unable to open type definition storage";
        let static_storage_config = Self::static_storage_config(config, static_config);

        let static_storage = match
                <<S::StaticStorage as iceoryx2_cal::static_storage::StaticStorage>::Builder as NamedConceptBuilder::<S::StaticStorage>>::new(name).config(&static_storage_config).open(Duration::ZERO) {
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
                            "{msg} since the type definition file is not yet initialized.");
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

        static_storage.release_ownership();

        Ok(Some(Self {
            storage: static_storage,
            path_hint: *static_storage_config.get_path_hint(),
        }))
    }

    fn static_storage_config(
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
        let origin = "TypeDefinitionStorage::service_resource_directory()";
        let mut root = config.global.service_dir();
        let id = fatal_panic!(from origin,
               when Path::new(static_config.unique_service_id().value().to_string().as_bytes()),
               "This should never happen! The service id is always a valid path name.");
        fatal_panic!(from origin,
                when root.add_path_entry(&id),
                "This should never happen! The full service directory is too long. A shorter iceoryx2 root path might solve the issue.");
        root
    }
}
