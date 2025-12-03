// Copyright (c) 2020 - 2021 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_FUNCTION_HPP
#define IOX2_BB_FUNCTION_HPP

#include "iox2/bb/detail/storable_function.hpp"

namespace iox2 {
namespace bb {
constexpr uint64_t DEFAULT_FUNCTION_CAPACITY { 128U };

/// @brief A static memory replacement for std::function
///
///        Allows storing a callable with a given signature if its size does not exceed a limit.
///        This limit can be adjusted by changing the Bytes parameter.
///        The iox2::bb::Function objects own everything needed to invoke the underlying callable and can be safely
///        stored. They also support copy and move semantics in natural way by copying or moving the underlying
///        callable.
///
///        Similarly to std::function, they cannot be stored in Shared Memory to be invoked in a different process.
///
///        For the API see storable_function.
///
/// @tparam Signature The signature of the callable to be stored, e.g. int (char, void*).
/// @tparam Capacity The static storage capacity available to store a callable in bytes.
///
/// @note  If the static storage is insufficient to store the callable we get a compile time error.
///

template <typename Signature, uint64_t Capacity = DEFAULT_FUNCTION_CAPACITY>
using Function = detail::storable_function<Capacity, Signature>;

} // namespace bb
} // namespace iox2

#endif // IOX2_BB_FUNCTION_HPP
