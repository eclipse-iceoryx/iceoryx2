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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_CONFIGURATION_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_CONFIGURATION_HPP

#include "iox2/legacy/error_reporting/configuration.hpp"

namespace iox2 {
namespace legacy {
namespace er {

// Specialize to change the checks (and other options if needed) at compile time.
// this can later also be done depending on a #define to select a header
// but we should avoid to have a #define for each option.
template <>
struct ConfigurationParameters<ConfigurationTag> {
    static constexpr bool CHECK_ASSERT { true }; /// @todo iox-#1032 deactive for release builds
};

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_CONFIGURATION_HPP
