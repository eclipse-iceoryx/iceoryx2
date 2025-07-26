// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use crate::{config, node::NodeId};
use core::fmt::Debug;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_cal::named_concept::{NamedConceptConfiguration, NamedConceptMgmt};

pub(crate) fn dynamic_config_storage_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::DynamicStorage as NamedConceptMgmt>::Configuration {
    <<Service::DynamicStorage as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.dynamic_config_storage_suffix)
        .path_hint(global_config.global.root_path())
}

pub(crate) fn static_config_storage_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::StaticStorage as NamedConceptMgmt>::Configuration {
    let origin = "static_config_storage_config";
    let msg = "Unable to generate static config storage directory";
    let mut path_hint = global_config.global.root_path().clone();
    fatal_panic!(from origin, when path_hint.add_path_entry(&global_config.global.service.directory),
            "{} since the combination of root directory and service directory entry result in an invalid directory \"{}{}\".",
            msg, path_hint, global_config.global.service.directory);

    <<Service::StaticStorage as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.static_config_storage_suffix)
        .path_hint(&path_hint)
}

pub(crate) fn connection_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::Connection as NamedConceptMgmt>::Configuration {
    <<Service::Connection as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.connection_suffix)
        .path_hint(global_config.global.root_path())
}

pub(crate) fn event_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::Event as NamedConceptMgmt>::Configuration {
    <<Service::Event as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.event_connection_suffix)
        .path_hint(global_config.global.root_path())
}

pub(crate) fn data_segment_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::SharedMemory as NamedConceptMgmt>::Configuration {
    <<Service::SharedMemory as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.data_segment_suffix)
        .path_hint(global_config.global.root_path())
}

pub(crate) fn resizable_data_segment_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::ResizableSharedMemory as NamedConceptMgmt>::Configuration {
    <<Service::ResizableSharedMemory as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.data_segment_suffix)
        .path_hint(global_config.global.root_path())
}

pub(crate) fn node_monitoring_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::Monitoring as NamedConceptMgmt>::Configuration {
    <<Service::Monitoring as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.node.monitor_suffix)
        .path_hint(&global_config.global.node_dir())
}

pub(crate) fn node_details_path(
    global_config: &config::Config,
    node_id: &NodeId,
) -> iceoryx2_bb_system_types::path::Path {
    let origin = "node_details_path";
    let mut path = global_config.global.node_dir();
    fatal_panic!(from origin, when path.add_path_entry(&node_id.as_file_name().into()),
                    "The node path exceeds the maximum path length.");
    path
}

pub(crate) fn node_details_config<Service: crate::service::Service>(
    global_config: &config::Config,
    node_id: &NodeId,
) -> <Service::StaticStorage as NamedConceptMgmt>::Configuration {
    <<Service::StaticStorage as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.node.static_config_suffix)
        .path_hint(&node_details_path(global_config, node_id))
}

pub(crate) fn service_tag_config<Service: crate::service::Service>(
    global_config: &config::Config,
    node_id: &NodeId,
) -> <Service::StaticStorage as NamedConceptMgmt>::Configuration {
    <<Service::StaticStorage as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.node.service_tag_suffix)
        .path_hint(&node_details_path(global_config, node_id))
}

pub(crate) fn blackboard_mgmt_config<
    Service: crate::service::Service,
    T: ZeroCopySend + Send + Sync + Debug + 'static,
>(
    global_config: &config::Config,
) -> <Service::BlackboardMgmt<T> as NamedConceptMgmt>::Configuration {
    <<Service::BlackboardMgmt<T> as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.blackboard_mgmt_suffix)
        .path_hint(global_config.global.root_path())
}

pub(crate) fn blackboard_data_config<Service: crate::service::Service>(
    global_config: &config::Config,
) -> <Service::BlackboardPayload as NamedConceptMgmt>::Configuration {
    <<Service::BlackboardPayload as NamedConceptMgmt>::Configuration>::default()
        .prefix(&global_config.global.prefix)
        .suffix(&global_config.global.service.blackboard_data_suffix)
        .path_hint(global_config.global.root_path())
}
