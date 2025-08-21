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

#ifndef IOX2_NODE_NAME_HPP
#define IOX2_NODE_NAME_HPP

#include "iox/expected.hpp"
#include "iox/string.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/semantic_string.hpp"

namespace iox2 {
namespace internal {
template <ServiceType>
auto list_callback(iox2_node_state_e,
                   iox2_node_id_ptr,
                   const char* executable,
                   iox2_node_name_ptr,
                   iox2_config_ptr,
                   iox2_callback_context) -> iox2_callback_progression_e;
}
class NodeName;

/// Non-owning view of a [`NodeName`].
class NodeNameView {
  public:
    NodeNameView(NodeNameView&&) = default;
    NodeNameView(const NodeNameView&) = default;
    auto operator=(NodeNameView&&) -> NodeNameView& = default;
    auto operator=(const NodeNameView&) -> NodeNameView& = default;
    ~NodeNameView() = default;

    /// Returns a [`iox::string`] containing the [`NodeName`].
    auto to_string() const -> iox::string<IOX2_NODE_NAME_LENGTH>;

    /// Creates a copy of the corresponding [`NodeName`] and returns it.
    auto to_owned() const -> NodeName;

  private:
    template <ServiceType>
    friend class Node;
    friend class NodeName;
    template <ServiceType>
    friend auto internal::list_callback(iox2_node_state_e,
                                        iox2_node_id_ptr,
                                        const char* executable,
                                        iox2_node_name_ptr,
                                        iox2_config_ptr,
                                        iox2_callback_context) -> iox2_callback_progression_e;

    explicit NodeNameView(iox2_node_name_ptr ptr);
    iox2_node_name_ptr m_ptr = nullptr;
};

/// Represent the name for a [`Node`].
class NodeName {
  public:
    NodeName(NodeName&&) noexcept;
    auto operator=(NodeName&&) noexcept -> NodeName&;
    NodeName(const NodeName&);
    auto operator=(const NodeName&) -> NodeName&;
    ~NodeName();

    /// Creates a [`NodeNameView`]
    auto as_view() const -> NodeNameView;

    /// Creates a new [`NodeName`].
    /// If the provided name does not contain a valid [`NodeName`] it will return a
    /// [`SemanticStringError`] otherwise the [`NodeName`].
    static auto create(const char* value) -> iox::expected<NodeName, SemanticStringError>;

    /// Returns a [`iox::string`] containing the [`NodeName`].
    auto to_string() const -> iox::string<IOX2_NODE_NAME_LENGTH>;

  private:
    friend class NodeBuilder;
    friend class NodeNameView;

    explicit NodeName(iox2_node_name_h handle);
    void drop() noexcept;
    static auto create_impl(const char* value, size_t value_len) -> iox::expected<NodeName, SemanticStringError>;

    iox2_node_name_h m_handle = nullptr;
};
} // namespace iox2

#endif
