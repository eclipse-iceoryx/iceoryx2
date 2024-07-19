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

#include "internal/iceoryx2.hpp"
#include "iox/expected.hpp"
#include "iox/string.hpp"
#include "iox2/iceoryx2_settings.hpp"
#include "semantic_string.hpp"

namespace iox2 {
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
    auto to_string() const -> iox::string<NODE_NAME_LENGTH>;

    /// Creates a copy of the corresponding [`NodeName`] and returns it.
    auto to_owned() const -> NodeName;

  private:
    friend class NodeName;
    explicit NodeNameView(iox2_node_name_ptr handle);
    iox2_node_name_ptr m_ptr;
};

/// Represent the name for a [`Node`].
class NodeName {
  public:
    NodeName(NodeName&&) noexcept;
    auto operator=(NodeName&&) noexcept -> NodeName&;
    NodeName(const NodeName&);
    auto operator=(const NodeName&) -> NodeName&;
    ~NodeName();

    /// Creates a new [`NodeName`].
    /// If the provided name does not contain a valid [`NodeName`] it will return a
    /// [`SemanticStringError`] otherwise the [`NodeName`].
    static auto create(const char* value) -> iox::expected<NodeName, SemanticStringError>;

    /// Returns a [`iox::string`] containing the [`NodeName`].
    auto to_string() const -> iox::string<NODE_NAME_LENGTH>;

  private:
    friend class NodeBuilder;
    template <ServiceType>
    friend class Node;
    friend class NodeNameView;

    explicit NodeName(iox2_node_name_h handle);
    void drop() noexcept;
    static auto create_impl(const char* value, size_t value_len) -> iox::expected<NodeName, SemanticStringError>;

    iox2_node_name_h m_handle;
};
} // namespace iox2

#endif
