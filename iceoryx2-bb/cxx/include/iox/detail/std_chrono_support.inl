// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_UTILITY_STD_CHRONO_SUPPORT_INL
#define IOX_HOOFS_UTILITY_STD_CHRONO_SUPPORT_INL

#include "iox/std_chrono_support.hpp"

namespace iox
{

inline units::Duration
FromImpl<std::chrono::nanoseconds, units::Duration>::fromImpl(const std::chrono::nanoseconds& value) noexcept
{
    return units::Duration::fromNanoseconds(value.count());
}
inline units::Duration
FromImpl<std::chrono::microseconds, units::Duration>::fromImpl(const std::chrono::microseconds& value) noexcept
{
    return units::Duration::fromMicroseconds(value.count());
}

inline units::Duration
FromImpl<std::chrono::milliseconds, units::Duration>::fromImpl(const std::chrono::milliseconds& value) noexcept
{
    return units::Duration::fromMilliseconds(value.count());
}

inline units::Duration
FromImpl<std::chrono::seconds, units::Duration>::fromImpl(const std::chrono::seconds& value) noexcept
{
    return units::Duration::fromSeconds(value.count());
}

} // namespace iox

#endif // IOX_HOOFS_UTILITY_STD_CHRONO_SUPPORT_INL
