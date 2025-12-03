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

#include "iox2/legacy/cli/option_manager.hpp"

namespace iox2 {
namespace legacy {
namespace cli {
OptionManager::OptionManager(const OptionDescription_t& programDescription,
                             const bb::function<void()>& onFailureCallback)
    : m_optionSet { programDescription, onFailureCallback } {
}

void OptionManager::populateDefinedOptions(const char*& binaryName, int argc, char** argv, const uint64_t argcOffset) {
    auto options = m_parser.parse(m_optionSet, argc, argv, argcOffset);

    for (const auto& assignment : m_assignments) {
        assignment(options);
    }

    binaryName = options.binaryName();
}

OptionName_t OptionManager::getLookupName(const char shortName, const OptionName_t& name) noexcept {
    if (shortName == NO_SHORT_OPTION) {
        return OptionName_t { TruncateToCapacity, &shortName, 1 };
    }

    return name;
}

} // namespace cli
} // namespace legacy
} // namespace iox2
