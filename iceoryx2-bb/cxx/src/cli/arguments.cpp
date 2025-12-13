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

#include "iox2/legacy/cli/arguments.hpp"

namespace iox2 {
namespace legacy {
namespace cli {
const char* Arguments::binaryName() const noexcept {
    return m_binaryName;
}

bool Arguments::isSwitchSet(const OptionName_t& switchName) const noexcept {
    for (const auto& a : m_arguments) {
        if (a.isSwitch && a.hasOptionName(switchName)) {
            return true;
        }
    }
    return false;
}
} // namespace cli
} // namespace legacy
} // namespace iox2
