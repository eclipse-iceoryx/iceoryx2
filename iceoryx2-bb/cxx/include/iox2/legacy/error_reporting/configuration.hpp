// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_CONFIGURATION_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_CONFIGURATION_HPP

#include <type_traits>

// ***
// * Configure active checks and other compile time parameters
// ***

namespace iox2 {
namespace legacy {
namespace er {

// tag type that can be used to override the configuration in a custom implementation
struct ConfigurationTag { };

// can be specialized here to change parameters at compile time
template <typename T>
struct ConfigurationParameters {
    static_assert(std::is_same<T, ConfigurationTag>::value, "Incorrect configuration tag type");

    static constexpr bool CHECK_ASSERT { true }; /// @todo iox-#1032 deactive for release builds
};

// used by the API to obtain the compile time parameters
using Configuration = ConfigurationParameters<ConfigurationTag>;

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_CONFIGURATION_HPP
