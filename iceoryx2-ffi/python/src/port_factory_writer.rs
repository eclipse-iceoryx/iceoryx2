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

use iceoryx2::service::builder::CustomKeyMarker;
use iceoryx2_log::fatal_panic;
use pyo3::prelude::*;

use crate::error::WriterCreateError;
use crate::parc::Parc;
use crate::port_factory_blackboard::PortFactoryBlackboardType;
use crate::type_storage::TypeStorage;
use crate::writer::{Writer, WriterType};

type IpcPortFactoryWriter<'a> = iceoryx2::service::port_factory::writer::PortFactoryWriter<
    'a,
    crate::IpcService,
    CustomKeyMarker,
>;
type LocalPortFactoryWriter<'a> = iceoryx2::service::port_factory::writer::PortFactoryWriter<
    'a,
    crate::LocalService,
    CustomKeyMarker,
>;

pub(crate) enum PortFactoryWriterType {
    Ipc(Parc<IpcPortFactoryWriter<'static>>),
    Local(Parc<LocalPortFactoryWriter<'static>>),
}

#[pyclass]
/// Factory to create a new `Writer` port/endpoint for `MessagingPattern::Blackboard`
/// based communication.
pub struct PortFactoryWriter {
    factory: Parc<PortFactoryBlackboardType>,
    value: PortFactoryWriterType,
    key_type_storage: TypeStorage,
}

impl PortFactoryWriter {
    pub(crate) fn new(
        factory: Parc<PortFactoryBlackboardType>,
        key_type_storage: TypeStorage,
    ) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryBlackboardType::Ipc(Some(v)) => PortFactoryWriterType::Ipc(unsafe {
                    Parc::new(core::mem::transmute::<
                        IpcPortFactoryWriter<'_>,
                        IpcPortFactoryWriter<'static>,
                    >(v.writer_builder()))
                }),
                PortFactoryBlackboardType::Local(Some(v)) => PortFactoryWriterType::Local(unsafe {
                    Parc::new(core::mem::transmute::<
                        LocalPortFactoryWriter<'_>,
                        LocalPortFactoryWriter<'static>,
                    >(v.writer_builder()))
                }),
                _ => {
                    fatal_panic!(from "PortFactoryWriter::new()", "Accessing a deleted PortFactoryBlackboard.")
                }
            },
            key_type_storage,
        }
    }
}

#[pymethods]
impl PortFactoryWriter {
    /// Creates a new `Writer` or returns a `WriterCreateError` on failure.
    pub fn create(&self) -> PyResult<Writer> {
        let _guard = self.factory.lock();
        match &self.value {
            PortFactoryWriterType::Ipc(v) => {
                let this = (*v.lock()).clone();
                Ok(Writer {
                    value: Parc::new(WriterType::Ipc(Some(
                        this.create()
                            .map_err(|e| WriterCreateError::new_err(format!("{e:?}")))?,
                    ))),
                    key_type_storage: self.key_type_storage.clone(),
                })
            }
            PortFactoryWriterType::Local(v) => {
                let this = (*v.lock()).clone();
                Ok(Writer {
                    value: Parc::new(WriterType::Local(Some(
                        this.create()
                            .map_err(|e| WriterCreateError::new_err(format!("{e:?}")))?,
                    ))),
                    key_type_storage: self.key_type_storage.clone(),
                })
            }
        }
    }
}
