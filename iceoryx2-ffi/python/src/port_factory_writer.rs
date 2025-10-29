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
use pyo3::prelude::*;

use crate::parc::Parc;
use crate::port_factory_blackboard::PortFactoryBlackboardType;
use crate::type_storage::TypeStorage;

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
    key_type_details: TypeStorage,
}

impl PortFactoryWriter {
    pub(crate) fn new(
        factory: Parc<PortFactoryBlackboardType>,
        key_type_details: TypeStorage,
    ) -> Self {
        Self {
            factory: factory.clone(),
            value: match &*factory.lock() {
                PortFactoryBlackboardType::Ipc(v) => PortFactoryWriterType::Ipc(unsafe {
                    Parc::new(core::mem::transmute::<
                        IpcPortFactoryWriter<'_>,
                        IpcPortFactoryWriter<'static>,
                    >(v.writer_builder()))
                }),
                PortFactoryBlackboardType::Local(v) => PortFactoryWriterType::Local(unsafe {
                    Parc::new(core::mem::transmute::<
                        LocalPortFactoryWriter<'_>,
                        LocalPortFactoryWriter<'static>,
                    >(v.writer_builder()))
                }),
            },
            key_type_details,
        }
    }

    fn clone_ipc(&self, value: IpcPortFactoryWriter<'static>) -> Self {
        Self {
            factory: self.factory.clone(),
            value: PortFactoryWriterType::Ipc(Parc::new(value)),
            key_type_details: self.key_type_details.clone(),
        }
    }

    fn clone_local(&self, value: LocalPortFactoryWriter<'static>) -> Self {
        Self {
            factory: self.factory.clone(),
            value: PortFactoryWriterType::Local(Parc::new(value)),
            key_type_details: self.key_type_details.clone(),
        }
    }
}
