// Copyright (c) 2020 - 2021 by Apex.AI Inc. All rights reserved.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#ifndef IOX_HOOFS_FUNCTIONAL_FUNCTION_HPP
#define IOX_HOOFS_FUNCTIONAL_FUNCTION_HPP

#include "iox/detail/storable_function.hpp"

namespace iox
{
constexpr uint64_t DEFAULT_FUNCTION_CAPACITY{128U};

/// @brief A static memory replacement for std::function
///        Allows storing a callable with a given signature if its size does not exceed a limit.
///        This limit can be adjusted by changing the Bytes parameter.
///        In contrast to iox::function_ref iox::function objects own everything needed
///        to invoke the underlying callable and can be safely stored.
///        They also support copy and move semantics in natural way
///        by copying or moving the underlying callable.
///
///        Similarly to std::function, they cannot be stored in Shared Memory
///        to be invoked in a different process.
///
///        For the API see storable_function.
///
/// @tparam Signature The signature of the callable to be stored, e.g. int (char, void*).
/// @tparam Capacity The static storage capacity available to store a callable in bytes.
///
/// @note  If the static storage is insufficient to store the callable we get a compile time error.
///

template <typename Signature, uint64_t Capacity = DEFAULT_FUNCTION_CAPACITY>
using function = storable_function<Capacity, Signature>;

} // namespace iox

#endif // IOX_HOOFS_FUNCTIONAL_FUNCTION_HPP
