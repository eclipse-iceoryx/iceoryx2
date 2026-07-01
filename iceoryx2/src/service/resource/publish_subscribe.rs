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

use crate::node::SharedNode;
use crate::service;
use crate::service::resource::ServiceResource;
use core::marker::PhantomData;
use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_flatbuffers::{TypeName, find_best_fitting_schema_file};
use iceoryx2_bb_posix::{directory::Directory, file::File};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_log::{fail, warn};

const TYPE_DEFINITION_FILE: FileName =
    unsafe { FileName::new_unchecked_const(b"payload.type_definition") };

pub struct PublishSubscribeResourceConfig<ServiceType: service::Service> {
    pub(crate) use_type_definition: bool,
    pub(crate) schema_path: Option<FilePath>,
    pub(crate) type_name: TypeName,
    pub(crate) shared_node: SharedNode<ServiceType>,
}

#[derive(Debug)]
pub struct PublishSubscribeResources<ServiceType: service::Service> {
    type_definition: Option<FilePath>,
    resource_directory: Path,
    has_ownership: AtomicBool,
    _service_type: PhantomData<ServiceType>,
}

impl<ServiceType: service::Service> Drop for PublishSubscribeResources<ServiceType> {
    fn drop(&mut self) {
        let origin = "PublishSubscribeResources::drop()";
        if self.has_ownership.load(Ordering::Relaxed) {
            if let Some(file) = self.type_definition {
                if let Err(e) = File::remove(&file) {
                    warn!(from origin,
                        "Failed to remove service type definition \"{file}\". [{e:?}]");
                }
            }

            if let Err(e) = Directory::remove(&self.resource_directory) {
                warn!(from origin,
                    "Unable to remove service resource directory \"{}\". [{e:?}]", self.resource_directory);
            }
        }
    }
}

impl<ServiceType: service::Service> Abandonable for PublishSubscribeResources<ServiceType> {
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        this.has_ownership.store(false, Ordering::Relaxed);
    }
}

impl<ServiceType: service::Service> ServiceResource for PublishSubscribeResources<ServiceType> {
    type Config = PublishSubscribeResourceConfig<ServiceType>;

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
    }

    fn create(
        static_config: &crate::service::static_config::StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, crate::service::builder::ServiceCreateError> {
        let directory = Self::create_service_resource_directory(
            resource_config.shared_node.config(),
            static_config,
        )?;
        directory.acquire_ownership();

        if resource_config.use_type_definition {
            let schema_path = Self::schema_path(resource_config);
            let schema_dest =
                Self::type_definition_path(resource_config.shared_node.config(), static_config);

            File::copy(&schema_path, &schema_dest).unwrap();

            directory.release_ownership();

            Ok(Self {
                resource_directory: *directory.path(),
                type_definition: Some(schema_dest),
                has_ownership: AtomicBool::new(false),
                _service_type: PhantomData,
            })
        } else {
            Ok(Self {
                resource_directory: *directory.path(),
                type_definition: None,
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
            let schema_path = Self::schema_path(resource_config);
            let service_schema =
                Self::type_definition_path(resource_config.shared_node.config(), static_config);

            if File::compare(&schema_path, &service_schema).unwrap() {
                fail!(from origin, with service::builder::ServiceOpenError::IncompatiblePayload,
                "{msg} since the payload definition of the service {service_schema} is not the same as the one requested in {schema_path}. Both files must be identical!");
            }

            Ok(Self {
                resource_directory: Self::service_resource_directory(
                    resource_config.shared_node.config(),
                    static_config,
                ),
                has_ownership: AtomicBool::new(false),
                type_definition: Some(service_schema),
                _service_type: PhantomData,
            })
        } else {
            Ok(Self {
                resource_directory: Self::service_resource_directory(
                    resource_config.shared_node.config(),
                    static_config,
                ),
                has_ownership: AtomicBool::new(false),
                type_definition: None,
                _service_type: PhantomData,
            })
        }
    }

    fn remove_stale_resources(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) {
        let type_definition_path = Self::type_definition_path(config, static_config);
        let resource_path = Self::service_resource_directory(config, static_config);

        File::remove(&type_definition_path).unwrap();
        Directory::remove(&resource_path).unwrap();
    }
}

impl<ServiceType: service::Service> PublishSubscribeResources<ServiceType> {
    fn type_definition_path(
        config: &crate::config::Config,
        static_config: &crate::service::static_config::StaticConfig,
    ) -> FilePath {
        let dir = Self::service_resource_directory(config, static_config);
        FilePath::from_path_and_file(&dir, &TYPE_DEFINITION_FILE).unwrap()
    }

    fn schema_path(resource_config: &PublishSubscribeResourceConfig<ServiceType>) -> FilePath {
        let flatbuffer_schema_path = match resource_config
            .shared_node
            .config()
            .global
            .service
            .flatbuffer_schema_path
        {
            Some(p) => p,
            None => {
                todo!()
            }
        };

        match resource_config.schema_path {
            Some(file_path) => {
                if file_path.path().is_absolute() {
                    file_path
                } else {
                    let mut path = flatbuffer_schema_path;
                    path.add_path_entry(&file_path.into()).unwrap();
                    unsafe { FilePath::new_unchecked(path.as_bytes()) }
                }
            }
            None => {
                find_best_fitting_schema_file(&resource_config.type_name, &flatbuffer_schema_path)
                    .unwrap()
                    .unwrap()
            }
        }
    }
}
