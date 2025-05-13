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

use iceoryx2_bb_log::fail;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::event::NamedConceptMgmt;
use iceoryx2_cal::named_concept::NamedConceptListError;
use iceoryx2_cal::named_concept::NamedConceptRemoveError;
use iceoryx2_cal::zero_copy_connection::{ZeroCopyConnection, ZeroCopyPortRemoveError};

use crate::config;
use crate::service;
use crate::service::config_scheme::data_segment_config;
use crate::service::naming_scheme::data_segment_name;

use super::config_scheme::connection_config;
use super::naming_scheme::extract_receiver_port_id_from_connection;
use super::naming_scheme::extract_sender_port_id_from_connection;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum RemovePortFromAllConnectionsError {
    CleanupRaceDetected,
    InsufficientPermissions,
    VersionMismatch,
    InternalError,
}

pub(crate) unsafe fn remove_data_segment_of_port<Service: service::Service>(
    port_id: u128,
    config: &config::Config,
) -> Result<(), NamedConceptRemoveError> {
    let origin = format!(
        "remove_data_segment_of_port::<{}>::({:?})",
        core::any::type_name::<Service>(),
        port_id
    );

    fail!(from origin, when <Service::SharedMemory as NamedConceptMgmt>::remove_cfg(
            &data_segment_name(port_id),
            &data_segment_config::<Service>(config),
        ), "Unable to remove the ports ({port_id}) data segment."
    );

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
        Ok(()) => Ok(()),
        Err(ZeroCopyPortRemoveError::DoesNotExist) => {
            fail!(from origin,
                with RemovePortFromAllConnectionsError::CleanupRaceDetected,
                "{} since the connection ({:?}) no longer exists! This could indicate a race in the node cleanup algorithm or that the underlying resources were removed manually.",
                msg, connection);
        }
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
    }
}

pub(crate) unsafe fn remove_sender_port_from_all_connections<Service: service::Service>(
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
        if let Some(sender_port_id) = extract_sender_port_id_from_connection(&connection) {
            if sender_port_id == port_id {
                let result = handle_port_remove_error(
                    Service::Connection::remove_sender(&connection, &connection_config),
                    &origin,
                    msg,
                    &connection,
                );

                if ret_val.is_ok() {
                    ret_val = result;
                }
            }
        }
    }

    ret_val
}

pub(crate) unsafe fn remove_receiver_port_from_all_connections<Service: service::Service>(
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
        if let Some(receiver_port_id) = extract_receiver_port_id_from_connection(&connection) {
            if receiver_port_id == port_id {
                let result = handle_port_remove_error(
                    Service::Connection::remove_receiver(&connection, &connection_config),
                    &origin,
                    msg,
                    &connection,
                );

                if ret_val.is_ok() {
                    ret_val = result;
                }
            }
        }
    }

    ret_val
}
