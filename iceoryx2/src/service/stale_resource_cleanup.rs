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

use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::event::NamedConceptMgmt;
use iceoryx2_cal::named_concept::NamedConceptListError;
use iceoryx2_cal::named_concept::NamedConceptRemoveError;
use iceoryx2_cal::zero_copy_connection::{ZeroCopyConnection, ZeroCopyPortRemoveError};
use iceoryx2_log::{debug, fail};

use crate::config;
use crate::identifiers::UniqueNodeId;
use crate::service;
use crate::service::Service;
use crate::service::config_scheme::port_tag_config;
use crate::service::config_scheme::service_tag_config;
use crate::service::config_scheme::static_config_storage_config;
use crate::service::config_scheme::{data_segment_config, resizable_data_segment_config};
use crate::service::naming_scheme::data_segment_name;
use crate::service::naming_scheme::static_config_name;
use crate::service::service_hash::ServiceHash;

use super::config_scheme::connection_config;
use super::naming_scheme::extract_receiver_port_id_from_connection;
use super::naming_scheme::extract_sender_port_id_from_connection;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum RemovePortFromAllConnectionsError {
    InsufficientPermissions,
    Interrupt,
    VersionMismatch,
    InternalError,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum RemoveStalePortResourcesError {
    InsufficientPermissions,
    Interrupt,
    VersionMismatch,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortRemoveTagError {
    Interrupt,
    AlreadyRemoved,
    InternalError,
    InsufficientPermissions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRemoveTagError {
    Interrupt,
    AlreadyRemoved,
    InternalError,
    InsufficientPermissions,
}

pub struct CleanupFailure;

pub unsafe fn remove_static_service_config<S: Service>(
    config: &config::Config,
    service_hash: &ServiceHash,
) -> Result<bool, NamedConceptRemoveError> {
    let msg = "Unable to remove static service config";
    let origin = "Service::remove_static_service_config()";
    let name = static_config_name(service_hash);
    let static_storage_config = static_config_storage_config::<S>(config);

    match unsafe {
        <S::StaticStorage as NamedConceptMgmt>::remove_cfg(&name, &static_storage_config)
    } {
        Ok(v) => Ok(v),
        Err(e) => {
            fail!(from origin, with e, "{msg} due to ({:?}).", e);
        }
    }
}

pub fn remove_sender_connection_and_data_segment<S: Service>(
    id: u128,
    config: &config::Config,
    origin: &str,
    port_name: &str,
) -> Result<(), CleanupFailure> {
    unsafe { remove_sender_port_from_all_connections::<S>(id, config) }.map_err(|e| {
        debug!(from origin,
            "Failed to remove the {} ({:?}) from all of its connections ({:?}).",
            port_name, id, e);
        CleanupFailure
    })?;

    unsafe { remove_data_segment_of_port::<S>(id, config) }.map_err(|e| {
        debug!(from origin,
            "Failed to remove the {} ({:?}) data segment ({:?}).",
            port_name, id, e);
        CleanupFailure
    })?;

    Ok(())
}

pub fn remove_sender_and_receiver_connections_and_data_segment<S: Service>(
    id: u128,
    config: &config::Config,
    origin: &str,
    port_name: &str,
) -> Result<(), CleanupFailure> {
    remove_sender_connection_and_data_segment::<S>(id, config, origin, port_name)?;
    unsafe { remove_receiver_port_from_all_connections::<S>(id, config) }.map_err(|e| {
        debug!(from origin,
                "Failed to remove the {} ({:?}) from all of its incoming connections ({:?}).",
                port_name, id, e);
        CleanupFailure
    })?;

    Ok(())
}

pub fn remove_service_tag<S: Service>(
    node_id: &UniqueNodeId,
    service_hash: &ServiceHash,
    config: &config::Config,
) -> Result<(), ServiceRemoveTagError> {
    let msg = "Unable to remove the service's tag for the node";
    let origin = format!(
        "remove_service_tag<{}>({:?}, service_hash: {:?})",
        core::any::type_name::<S>(),
        node_id,
        service_hash
    );

    match unsafe {
        <S::StaticStorage as NamedConceptMgmt>::remove_cfg(
            &service_hash.0.into(),
            &service_tag_config::<S>(config, node_id),
        )
    } {
        Ok(true) => Ok(()),
        Ok(false) => {
            fail!(from origin, with ServiceRemoveTagError::AlreadyRemoved,
                    "The service's tag for the node was already removed. This may indicate a corrupted system!");
        }
        Err(NamedConceptRemoveError::InternalError) => {
            fail!(from origin, with ServiceRemoveTagError::InternalError,
                "{msg} due to an internal error.");
        }
        Err(NamedConceptRemoveError::InsufficientPermissions) => {
            fail!(from origin, with ServiceRemoveTagError::InsufficientPermissions,
                "{msg} due to insufficient permissions.");
        }
        Err(NamedConceptRemoveError::Interrupt) => {
            fail!(from origin, with ServiceRemoveTagError::Interrupt,
                "{msg} since an interrupt signal was raised.");
        }
    }
}

pub fn remove_port_tag<S: Service>(
    node_id: &UniqueNodeId,
    port_id: u128,
    config: &config::Config,
) -> Result<(), PortRemoveTagError> {
    let msg = "Unable to remove the port's tag for the node";
    let origin = format!(
        "remove_port_tag<{}>({:?}, port_id: {:?})",
        core::any::type_name::<S>(),
        node_id,
        port_id
    );
    let name = FileName::new(port_id.to_string().as_bytes())
        .expect("A number is always a valid file name.");

    match unsafe {
        <S::StaticStorage as NamedConceptMgmt>::remove_cfg(
            &name,
            &port_tag_config::<S>(config, node_id),
        )
    } {
        Ok(true) => Ok(()),
        Ok(false) => {
            fail!(from origin, with PortRemoveTagError::AlreadyRemoved,
                    "The port's tag for the node was already removed. This may indicate a corrupted system!");
        }
        Err(NamedConceptRemoveError::InternalError) => {
            fail!(from origin, with PortRemoveTagError::InternalError,
                "{msg} due to an internal error.");
        }
        Err(NamedConceptRemoveError::InsufficientPermissions) => {
            fail!(from origin, with PortRemoveTagError::InsufficientPermissions,
                "{msg} due to insufficient permissions.");
        }
        Err(NamedConceptRemoveError::Interrupt) => {
            fail!(from origin, with PortRemoveTagError::Interrupt,
                "{msg} since an interrupt signal was raised.");
        }
    }
}

pub unsafe fn remove_data_segment_of_port<Service: service::Service>(
    port_id: u128,
    config: &config::Config,
) -> Result<(), NamedConceptRemoveError> {
    let origin = format!(
        "remove_data_segment_of_port::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );
    unsafe {
        fail!(from origin, when <Service::SharedMemory as NamedConceptMgmt>::remove_cfg(
                &data_segment_name(port_id),
                &data_segment_config::<Service>(config),
            ), "Unable to remove the ports ({port_id}) data segment."
        );

        fail!(from origin, when <Service::ResizableSharedMemory as NamedConceptMgmt>::remove_cfg(
                &data_segment_name(port_id),
                &resizable_data_segment_config::<Service>(config),
            ), "Unable to remove the ports ({port_id}) resizable data segment."
        );
    }
    Ok(())
}

pub unsafe fn remove_sender_port_from_all_connections<Service: service::Service>(
    port_id: u128,
    config: &config::Config,
) -> Result<(), RemovePortFromAllConnectionsError> {
    let origin = format!(
        "remove_sender_port_from_all_connections::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );
    let msg = "Unable to remove the sender port from all connections";

    let connection_config = connection_config::<Service>(config);
    let connection_list = connections::<Service>(&origin, msg, &connection_config)?;

    let mut ret_val = Ok(());
    for connection in connection_list {
        if let Some(sender_port_id) = extract_sender_port_id_from_connection(&connection)
            && sender_port_id == port_id
        {
            let result = handle_port_remove_error(
                unsafe { Service::Connection::remove_sender(&connection, &connection_config) },
                &origin,
                msg,
                &connection,
            );

            if ret_val.is_ok() {
                ret_val = result;
            }
        }
    }

    ret_val
}

pub unsafe fn remove_receiver_port_from_all_connections<Service: service::Service>(
    port_id: u128,
    config: &config::Config,
) -> Result<(), RemovePortFromAllConnectionsError> {
    let origin = format!(
        "remove_receiver_port_from_all_connections::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );
    let msg = "Unable to remove the receiver port from all connections";

    let connection_config = connection_config::<Service>(config);
    let connection_list = connections::<Service>(&origin, msg, &connection_config)?;

    let mut ret_val = Ok(());
    for connection in connection_list {
        if let Some(receiver_port_id) = extract_receiver_port_id_from_connection(&connection)
            && receiver_port_id == port_id
        {
            let result = handle_port_remove_error(
                unsafe { Service::Connection::remove_receiver(&connection, &connection_config) },
                &origin,
                msg,
                &connection,
            );

            if ret_val.is_ok() {
                ret_val = result;
            }
        }
    }

    ret_val
}

pub unsafe fn remove_stale_port_resources<Service: service::Service>(
    node_id: &UniqueNodeId,
    port_id: u128,
    config: &config::Config,
) -> Result<(), RemoveStalePortResourcesError> {
    let origin = format!(
        "remove_stale_port_resources<{}>({}, {:?})",
        core::any::type_name::<Service>(),
        port_id,
        config
    );
    let msg = "Failed to remove stale port resources";
    match unsafe { remove_data_segment_of_port::<Service>(port_id, config) } {
        Ok(()) => (),
        Err(NamedConceptRemoveError::InsufficientPermissions) => {
            fail!(from origin, with RemoveStalePortResourcesError::InsufficientPermissions,
                "{msg} due to insufficient permissions to remove the ports data segment.");
        }
        Err(NamedConceptRemoveError::InternalError) => {
            fail!(from origin, with RemoveStalePortResourcesError::InternalError,
                "{msg} due to an internal error while removing the ports data segment.");
        }
        Err(NamedConceptRemoveError::Interrupt) => {
            fail!(from origin, with RemoveStalePortResourcesError::Interrupt,
                "{msg} since an interrupt signal was raised.");
        }
    }

    for result in [
        unsafe { remove_sender_port_from_all_connections::<Service>(port_id, config) },
        unsafe { remove_receiver_port_from_all_connections::<Service>(port_id, config) },
    ] {
        match result {
            Ok(()) => (),
            Err(RemovePortFromAllConnectionsError::InsufficientPermissions) => {
                fail!(from origin, with RemoveStalePortResourcesError::InsufficientPermissions,
                    "{msg} due to insufficient permissions to remove the port from its connections.");
            }
            Err(RemovePortFromAllConnectionsError::VersionMismatch) => {
                fail!(from origin, with RemoveStalePortResourcesError::VersionMismatch,
                    "{msg} since the port could not be removed from its connection since iceoryx2 version does not match.");
            }
            Err(RemovePortFromAllConnectionsError::InternalError) => {
                fail!(from origin, with RemoveStalePortResourcesError::InternalError,
                    "{msg} due to an internal error while removing the port from its connection.");
            }
            Err(RemovePortFromAllConnectionsError::Interrupt) => {
                fail!(from origin, with RemoveStalePortResourcesError::InternalError,
                    "{msg} since an interrupt signal was raised.");
            }
        }
    }

    match remove_port_tag::<Service>(node_id, port_id, config) {
        Ok(()) | Err(PortRemoveTagError::AlreadyRemoved) => (),
        Err(PortRemoveTagError::InsufficientPermissions) => {
            fail!(from origin, with RemoveStalePortResourcesError::InsufficientPermissions,
                  "{msg} since the port tag could not be removed due to insufficient permissions.");
        }
        Err(PortRemoveTagError::Interrupt) => {
            fail!(from origin, with RemoveStalePortResourcesError::Interrupt,
                  "{msg} since an interrupt signal was received while removing the port tag.");
        }
        Err(PortRemoveTagError::InternalError) => {
            fail!(from origin, with RemoveStalePortResourcesError::InternalError,
                  "{msg} since an internal error occurred while removing the port tag.");
        }
    }

    Ok(())
}

fn connections<Service: service::Service>(
    origin: &str,
    msg: &str,
    config: &<Service::Connection as NamedConceptMgmt>::Configuration,
) -> Result<Vec<FileName>, RemovePortFromAllConnectionsError> {
    match <Service::Connection as NamedConceptMgmt>::list_cfg(config) {
        Ok(list) => Ok(list),
        Err(NamedConceptListError::InsufficientPermissions) => {
            fail!(from origin, with RemovePortFromAllConnectionsError::InsufficientPermissions,
                    "{} due to insufficient permissions to list all connections.", msg);
        }
        Err(NamedConceptListError::InternalError) => {
            fail!(from origin, with RemovePortFromAllConnectionsError::InternalError,
                "{} due to an internal error while listing all connections.", msg);
        }
    }
}

fn handle_port_remove_error(
    result: Result<(), ZeroCopyPortRemoveError>,
    origin: &str,
    msg: &str,
    connection: &FileName,
) -> Result<(), RemovePortFromAllConnectionsError> {
    match result {
        Ok(()) | Err(ZeroCopyPortRemoveError::DoesNotExist) => Ok(()),
        Err(ZeroCopyPortRemoveError::InsufficientPermissions) => {
            fail!(from origin,
                with RemovePortFromAllConnectionsError::InsufficientPermissions,
                "{} due to insufficient permissions to remove the connection ({:?}).",
                msg, connection);
        }
        Err(ZeroCopyPortRemoveError::VersionMismatch) => {
            fail!(from origin,
                with RemovePortFromAllConnectionsError::VersionMismatch,
                "{} since connection ({:?}) has a different iceoryx2 version.",
                msg, connection);
        }
        Err(ZeroCopyPortRemoveError::InternalError) => {
            fail!(from origin,
                with RemovePortFromAllConnectionsError::InternalError,
                "{} due to insufficient permissions to remove the connection ({:?}).",
                msg, connection);
        }
        Err(ZeroCopyPortRemoveError::Interrupt) => {
            fail!(from origin,
                with RemovePortFromAllConnectionsError::Interrupt,
                "{} since an interrupt signal was raised while removing the connection {:?}.", msg, connection);
        }
    }
}
