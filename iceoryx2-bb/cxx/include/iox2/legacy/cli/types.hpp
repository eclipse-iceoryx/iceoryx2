// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_CLI_TYPES_HPP
#define IOX2_BB_CLI_TYPES_HPP

#include "iox2/legacy/string.hpp"

#include <cstdint>

namespace iox2 {
namespace legacy {
namespace cli {
/// @brief defines the type of command line argument option
enum class OptionType : uint8_t {
    /// @brief option when provided is true
    Switch,
    /// @brief option with value which has to be provided
    Required,
    /// @brief option with value which can be provided
    Optional
};

static constexpr uint64_t MAX_OPTION_NAME_LENGTH = 32;
static constexpr uint64_t MAX_OPTION_ARGUMENT_LENGTH = 128;
static constexpr uint64_t MAX_OPTION_DESCRIPTION_LENGTH = 1024;
static constexpr uint64_t MAX_TYPE_NAME_LENGTH = 16;
static constexpr char NO_SHORT_OPTION = '\0';
static constexpr uint64_t MAX_NUMBER_OF_ARGUMENTS = 16;

using OptionName_t = string<MAX_OPTION_NAME_LENGTH>;
using OptionDescription_t = string<MAX_OPTION_DESCRIPTION_LENGTH>;
using Argument_t = string<MAX_OPTION_ARGUMENT_LENGTH>;
using TypeName_t = string<MAX_TYPE_NAME_LENGTH>;

} // namespace cli
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_CLI_TYPES_HPP
