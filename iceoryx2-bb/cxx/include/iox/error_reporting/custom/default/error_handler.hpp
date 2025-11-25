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

#ifndef IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_HPP
#define IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_HPP

#include "iox/polymorphic_handler.hpp"
#include "iox/static_lifetime_guard.hpp"

#include "iox/error_reporting/custom/default/default_error_handler.hpp"
#include "iox/error_reporting/custom/default/error_handler_interface.hpp"

namespace iox
{
namespace er
{

using ErrorHandler = iox::PolymorphicHandler<ErrorHandlerInterface, DefaultErrorHandler>;

using DefaultErrorHandlerGuard = iox::StaticLifetimeGuard<DefaultErrorHandler>;

} // namespace er
} // namespace iox

#endif // IOX_HOOFS_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_HPP
