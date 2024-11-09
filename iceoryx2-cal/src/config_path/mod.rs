// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

//! use iceoryx2_cal::config_path::config_dir;
//!
//! let config_dir = config_dir().unwrap();
//! println!("Config dir: {:?}", config_dir);

use dirs;

pub trait ConfigPathProvider {
    fn config_dir(&self) -> Option<std::path::PathBuf>;
}

pub struct DirsConfigPathProvider;

impl ConfigPathProvider for DirsConfigPathProvider {
    fn config_dir(&self) -> Option<std::path::PathBuf> {
        dirs::config_dir()
    }
}

pub fn config_dir() -> Option<std::path::PathBuf> {
    let provider = DirsConfigPathProvider;
    provider.config_dir()
}
