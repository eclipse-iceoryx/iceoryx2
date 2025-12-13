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
    return iox2_port_factory_blackboard_dynamic_config_number_of_readers(&m_handle);
}

auto DynamicConfigBlackboard::number_of_writers() const -> uint64_t {
    return iox2_port_factory_blackboard_dynamic_config_number_of_writers(&m_handle);
}

DynamicConfigBlackboard::DynamicConfigBlackboard(iox2_port_factory_blackboard_h handle)
    : m_handle { handle } {
}

void DynamicConfigBlackboard::list_readers(
    const iox2::bb::StaticFunction<CallbackProgression(ReaderDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_blackboard_dynamic_config_list_readers(
        &m_handle, internal::list_ports_callback<iox2_reader_details_ptr, ReaderDetailsView>, static_cast<void*>(&ctx));
}

void DynamicConfigBlackboard::list_writers(
    const iox2::bb::StaticFunction<CallbackProgression(WriterDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_blackboard_dynamic_config_list_writers(
        &m_handle, internal::list_ports_callback<iox2_writer_details_ptr, WriterDetailsView>, static_cast<void*>(&ctx));
}
} // namespace iox2
