// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX_HOOFS_DESIGN_BUILDER_HPP
#define IOX_HOOFS_DESIGN_BUILDER_HPP

/// @brief Macro which generates a setter method useful for a builder pattern.
/// @param[in] type the data type of the parameter
/// @param[in] name the name of the parameter
/// @param[in] defaultValue the default value of the parameter
/// @code
///   class MyBuilder {
///     IOX_BUILDER_PARAMETER(TypeA, NameB, ValueC)
///     // START generates the following code
///     public:
///       decltype(auto) NameB(TypeA const& value) &&
///       {
///           m_NameB = value;
///           return std::move(*this);
///       }
///
///       decltype(auto) NameB(TypeA&& value) &&
///       {
///           m_NameB = std::move(value);
///           return std::move(*this);
///       }
///
///     private:
///       TypeA m_NameB = ValueC;
///     // END
///   };
/// @endcode
// AXIVION Next Construct AutosarC++19_03-A16.0.1 this macro is used to prevent a large amount of boilerplate code
// which cannot be realized with templates or constexpr functions
// AXIVION Next Construct AutosarC++19_03-M16.0.6 brackets around macro parameter would lead in this case to compile
// time failures
// AXIVION Next Construct AutosarC++19_03-M16.3.1 multiple '##' operators are required to declare and use a variable
// AXIVION Next Construct AutosarC++19_03-M16.3.2 the '##' operator is required to be able to reduce boilerplate code
// NOLINTBEGIN(bugprone-macro-parentheses)
// NOLINTNEXTLINE(cppcoreguidelines-macro-usage)
#define IOX_BUILDER_PARAMETER(type, name, defaultValue)                                                                \
  public:                                                                                                              \
    decltype(auto) name(type const& value)&& noexcept                                                                  \
    {                                                                                                                  \
        m_##name = value;                                                                                              \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
    decltype(auto) name(type&& value)&& noexcept                                                                       \
    {                                                                                                                  \
        m_##name = std::move(value);                                                                                   \
        return std::move(*this);                                                                                       \
    }                                                                                                                  \
                                                                                                                       \
  private:                                                                                                             \
    type m_##name{defaultValue};
// NOLINTEND(bugprone-macro-parentheses)

#endif
