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

#include "iox2/dynamic_config_blackboard.hpp"

namespace iox2 {
auto DynamicConfigBlackboard::number_of_readers() const -> uint64_t {
    IOX_TODO();
}

auto DynamicConfigBlackboard::number_of_writers() const -> uint64_t {
    IOX_TODO();
}

DynamicConfigBlackboard::DynamicConfigBlackboard(/*iox2_port_factory_blackboard_h handle*/) {
    IOX_TODO();
}

void DynamicConfigBlackboard::list_readers(
    [[maybe_unused]] const iox::function<CallbackProgression(ReaderDetailsView)>& callback) const {
    IOX_TODO();
}

void DynamicConfigBlackboard::list_writers(
    [[maybe_unused]] const iox::function<CallbackProgression(WriterDetailsView)>& callback) const {
    IOX_TODO();
}
} // namespace iox2
