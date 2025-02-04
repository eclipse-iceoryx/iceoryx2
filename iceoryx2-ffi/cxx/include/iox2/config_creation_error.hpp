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

#ifndef IOX2_CONFIG_CREATION_ERROR_HPP
#define IOX2_CONFIG_CREATION_ERROR_HPP

#include <cstdint>

namespace iox2 {
/// Failures occurring while creating a new [`Config`] object with [`Config::from_file()`].
enum class ConfigCreationError : uint8_t {
    /// The config file could not be read.
    FailedToReadConfigFileContents,
    /// Parts of the config file could not be deserialized. Indicates some kind of syntax error.
    UnableToDeserializeContents,
    /// Insufficient permissions to open the config file.
    InsufficientPermissions,
    /// The provided config file does not exist
    ConfigFileDoesNotExist,
    /// The config file could not be opened due to an internal error
    UnableToOpenConfigFile,
};

} // namespace iox2

#endif
