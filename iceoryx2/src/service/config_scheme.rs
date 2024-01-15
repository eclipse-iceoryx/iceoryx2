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

use crate::config;
use iceoryx2_bb_container::{byte_string::FixedSizeByteString, semantic_string::SemanticString};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_cal::named_concept::{NamedConceptConfiguration, NamedConceptMgmt};

fn generate_default_config<T: NamedConceptConfiguration>(
    origin: &str,
    prefix: &str,
    suffix: &str,
    path_hint: &Path,
) -> T {
    let prefix = match FileName::new(prefix.as_bytes()) {
        Err(_) => {
            fatal_panic!(from origin, "The prefix \"{}\" provided by the config contains either invalid file name characters or is too long.",
                                       prefix);
        }
        Ok(v) => v,
    };

    let suffix = match FileName::new(suffix.as_bytes()) {
        Err(_) => {
            fatal_panic!(from origin, "The suffix \"{}\" provided by the config contains either invalid file name characters or is too long.",
                                       suffix);
        }
        Ok(v) => v,
    };

    T::default()
        .prefix(prefix)
        .suffix(suffix)
        .path_hint(*path_hint)
}

pub(crate) fn dynamic_config_storage_config<'config, Service: crate::service::Details<'config>>(
    global_config: &config::Config,
) -> <Service::DynamicStorage as NamedConceptMgmt>::Configuration {
    generate_default_config::<<Service::DynamicStorage as NamedConceptMgmt>::Configuration>(
        "dynamic_config_storage_config",
        &global_config.global.prefix,
        &global_config.global.service.dynamic_config_storage_suffix,
        &global_config.global.root_path(),
    )
}

pub(crate) fn static_config_storage_config<'config, Service: crate::service::Details<'config>>(
    global_config: &config::Config,
) -> <Service::StaticStorage as NamedConceptMgmt>::Configuration {
    let origin = "static_config_storage_config";
    let msg = "Unable to generate static config storage directory";
    let mut path_hint = global_config.global.root_path();
    let service_directory: FixedSizeByteString<{ FileName::max_len() }> = fatal_panic!(from origin,
            when FixedSizeByteString::from_bytes(global_config.global.service.directory.as_bytes()),
            "{} since the directory entry \"{}\" is invalid.",
            msg, global_config.global.service.directory);

    fatal_panic!(from origin, when path_hint.add_path_entry(&service_directory),
            "{} since the combination of root directory and service directory entry result in an invalid directory \"{}{}\".",
            msg, path_hint, service_directory);

    generate_default_config::<<Service::StaticStorage as NamedConceptMgmt>::Configuration>(
        "static_config_storage_config",
        &global_config.global.prefix,
        &global_config.global.service.static_config_storage_suffix,
        &path_hint,
    )
}

pub(crate) fn connection_config<'config, Service: crate::service::Details<'config>>(
    global_config: &config::Config,
) -> <Service::Connection as NamedConceptMgmt>::Configuration {
    generate_default_config::<<Service::Connection as NamedConceptMgmt>::Configuration>(
        "connection_config",
        &global_config.global.prefix,
        &global_config.global.service.connection_suffix,
        &global_config.global.root_path(),
    )
}

pub(crate) fn data_segment_config<'config, Service: crate::service::Details<'config>>(
    global_config: &config::Config,
) -> <Service::SharedMemory as NamedConceptMgmt>::Configuration {
    generate_default_config::<<Service::SharedMemory as NamedConceptMgmt>::Configuration>(
        "data_segment_config",
        &global_config.global.prefix,
        &global_config.global.service.publisher_data_segment_suffix,
        &global_config.global.root_path(),
    )
}
