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

#ifndef IOX2_DYNAMIC_CONFIG_BLACKBOARD_HPP
#define IOX2_DYNAMIC_CONFIG_BLACKBOARD_HPP

#include "iox/function.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/reader_details.hpp"
#include "iox2/writer_details.hpp"

#include <cstdint>

namespace iox2 {
/// The dynamic configuration of an [`MessagingPattern::Blackboard`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
class DynamicConfigBlackboard {
  public:
    DynamicConfigBlackboard(const DynamicConfigBlackboard&) = delete;
    DynamicConfigBlackboard(DynamicConfigBlackboard&&) = delete;
    auto operator=(const DynamicConfigBlackboard&) -> DynamicConfigBlackboard& = delete;
    auto operator=(DynamicConfigBlackboard&&) -> DynamicConfigBlackboard& = delete;
    ~DynamicConfigBlackboard() = default;

    /// Returns how many [`Reader`] ports are currently connected.
    auto number_of_readers() const -> uint64_t;

    /// Returns how many [`Writer`] ports are currently connected.
    auto number_of_writers() const -> uint64_t;

    /// Iterates over all [`Reader`]s and calls the callback with the
    /// corresponding [`ReaderDetailsView`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    void list_readers(const iox::function<CallbackProgression(ReaderDetailsView)>& callback) const;

    /// Iterates over all [`Writer`]s and calls the callback with the
    /// corresponding [`WriterDetailsView`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    void list_writers(const iox::function<CallbackProgression(WriterDetailsView)>& callback) const;

  private:
    template <ServiceType, typename>
    friend class PortFactoryBlackboard;

    explicit DynamicConfigBlackboard(/*iox2_port_factory_blackboard_h handle*/);

    // iox2_port_factory_blackboard_h m_handle = nullptr;
};
} // namespace iox2

#endif
