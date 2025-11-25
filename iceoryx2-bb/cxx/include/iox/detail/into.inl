// Copyright (c) 2021 - 2023 by Apex.AI Inc. All rights reserved.
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
#ifndef IOX_HOOFS_UTILITY_INTO_INL
#define IOX_HOOFS_UTILITY_INTO_INL

#include "iox/into.hpp"
#include "iox/type_traits.hpp"

namespace iox
{
template <typename SourceType, typename DestinationType>
inline constexpr typename detail::extract_into_type<DestinationType>::type_t from(const SourceType value) noexcept
{
    return FromImpl<SourceType, DestinationType>::fromImpl(value);
}

// AXIVION Next Construct AutosarC++19_03-A7.1.5 : 'auto' is only used for the generic implementation which will always result in a compile error
template <typename SourceType, typename DestinationType>
inline auto FromImpl<SourceType, DestinationType>::fromImpl(const SourceType&) noexcept
{
    static_assert(always_false_v<SourceType> && always_false_v<DestinationType>, "\n \
        Conversion for the specified types is not implemented!\n \
        Please specialize 'FromImpl::fromImpl'!\n \
        -------------------------------------------------------------------------\n \
        template <typename SourceType, typename DestinationType>\n \
        constexpr DestinationType FromImpl::fromImpl(const SourceType&) noexcept;\n \
        -------------------------------------------------------------------------");
}

// AXIVION Next Construct AutosarC++19_03-A15.5.3, AutosarC++19_03-A15.4.2, FaultDetection-NoexceptViolations : Intentional behavior. The library itself does not throw and on the implementation side a try-catch block can be used
template <typename DestinationType, typename SourceType>
inline constexpr typename detail::extract_into_type<DestinationType>::type_t into(const SourceType value) noexcept
{
    return from<SourceType, DestinationType>(value);
}
} // namespace iox

#endif // IOX_HOOFS_UTILITY_INTO_INL
