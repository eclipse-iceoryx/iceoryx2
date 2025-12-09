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

#ifndef IOX2_BUILDER_ADDENDUM_HPP
#define IOX2_BUILDER_ADDENDUM_HPP

#include "iox2/container/optional.hpp"

// NOLINTBEGIN(cppcoreguidelines-macro-usage)
// NOLINTBEGIN(bugprone-macro-parentheses)
#define IOX2_BUILDER_SWITCH(name)                                                                                      \
  public:                                                                                                              \
    auto name()&& noexcept -> decltype(auto) {                                                                         \
        m_##name = true;                                                                                               \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
  private:                                                                                                             \
    bool m_##name { false };

#define IOX2_BUILDER_PARAMETER(type, name, defaultValue)                                                               \
  public:                                                                                                              \
    auto name(type const& value)&& noexcept -> decltype(auto) {                                                        \
        m_##name = value;                                                                                              \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
    auto name(type&& value)&& noexcept -> decltype(auto) {                                                             \
        m_##name = std::move(value);                                                                                   \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
  private:                                                                                                             \
    type m_##name {                                                                                                    \
        defaultValue                                                                                                   \
    }

#define IOX2_BUILDER_OPTIONAL(type, name)                                                                              \
  public:                                                                                                              \
    auto name(type const& value)&& -> decltype(auto) {                                                                 \
        m_##name = iox2::container::Optional<type>(value);                                                             \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
    auto name(type&& value)&& -> decltype(auto) {                                                                      \
        m_##name = iox2::container::Optional<type>(std::move(value));                                                  \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
  private:                                                                                                             \
    iox2::container::Optional<type> m_##name
// NOLINTEND(bugprone-macro-parentheses)
// NOLINTEND(cppcoreguidelines-macro-usage)

#endif
