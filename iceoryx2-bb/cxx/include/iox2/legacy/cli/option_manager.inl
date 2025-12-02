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

#ifndef IOX2_BB_CLI_OPTION_MANAGER_INL
#define IOX2_BB_CLI_OPTION_MANAGER_INL

#include "iox2/legacy/cli/option_manager.hpp"

namespace iox2 {
namespace legacy {
namespace cli {
template <typename T>
inline T OptionManager::extractOptionArgumentValue(const Arguments& arguments,
                                                   const char shortName,
                                                   const OptionName_t& name,
                                                   const OptionType) {
    return arguments.get<T>(getLookupName(shortName, name))
        .or_else([this](auto&) { m_parser.printHelpAndExit(); })
        .value();
}

template <>
inline bool OptionManager::extractOptionArgumentValue(const Arguments& arguments,
                                                      const char shortName,
                                                      const OptionName_t& name,
                                                      const OptionType optionType) {
    if (optionType == OptionType::Switch) {
        return arguments.isSwitchSet(getLookupName(shortName, name));
    }

    return arguments.get<bool>(getLookupName(shortName, name))
        .or_else([this](auto&) { m_parser.printHelpAndExit(); })
        .value();
}

template <typename T>
// NOLINTJUSTIFICATION this is not a user facing API but hidden in a macro
// NOLINTNEXTLINE(readability-function-size)
inline T OptionManager::defineOption(T& referenceToMember,
                                     const char shortName,
                                     const OptionName_t& name,
                                     const OptionDescription_t& description,
                                     const OptionType optionType,
                                     T defaultArgumentValue) {
    constexpr bool IS_NO_SWITCH = false;
    m_optionSet.addOption(OptionWithDetails {
        { shortName, IS_NO_SWITCH, name, bb::into<bb::Lossy<Argument_t>>(convert::toString(defaultArgumentValue)) },
        description,
        optionType,
        { TypeInfo<T>::NAME } });

    m_assignments.emplace_back([this, &referenceToMember, optionType, shortName, name](Arguments& arguments) {
        referenceToMember = extractOptionArgumentValue<T>(arguments, shortName, name, optionType);
    });

    return defaultArgumentValue;
}
} // namespace cli
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_CLI_OPTION_MANAGER_HPP
