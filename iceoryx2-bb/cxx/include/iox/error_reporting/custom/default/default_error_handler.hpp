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

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_HANDLER_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_HANDLER_HPP

#include "iox/error_reporting/custom/default/error_handler_interface.hpp"
#include "iox/error_reporting/source_location.hpp"
#include "iox/error_reporting/types.hpp"

namespace iox
{
namespace er
{

/// @brief Defines the default reaction of dynamic error handling.
/// The default reaction is to do nothing apart from logging and termination on panic.
/// As this is common for all error handling of the given custom implementation, this happens in the
/// reporting API before the (polymorphic) custom behavior is invoked.
class DefaultErrorHandler : public ErrorHandlerInterface
{
  public:
    DefaultErrorHandler() = default;
    ~DefaultErrorHandler() override = default;
    DefaultErrorHandler(const DefaultErrorHandler&) = delete;
    DefaultErrorHandler(DefaultErrorHandler&&) = delete;
    DefaultErrorHandler& operator=(const DefaultErrorHandler&) = delete;
    DefaultErrorHandler& operator=(DefaultErrorHandler&&) = delete;

    /// @brief Defines the reaction on panic.
    void onPanic() override;

    /// @brief Defines the reaction on error.
    /// @param desc error descriptor
    void onReportError(ErrorDescriptor desc) override;

    /// @brief Defines the reaction on violation.
    /// @param desc error descriptor
    void onReportViolation(ErrorDescriptor desc) override;
};

} // namespace er
} // namespace iox

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_DEFAULT_ERROR_HANDLER_HPP
