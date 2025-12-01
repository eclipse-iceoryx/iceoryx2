// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_HPP
#define IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_HPP

#include "iox2/legacy/polymorphic_handler.hpp"
#include "iox2/legacy/static_lifetime_guard.hpp"

#include "iox2/legacy/error_reporting/custom/default/default_error_handler.hpp"
#include "iox2/legacy/error_reporting/custom/default/error_handler_interface.hpp"

namespace iox2 {
namespace legacy {
namespace er {

using ErrorHandler = iox2::legacy::PolymorphicHandler<ErrorHandlerInterface, DefaultErrorHandler>;

using DefaultErrorHandlerGuard = iox2::legacy::StaticLifetimeGuard<DefaultErrorHandler>;

} // namespace er
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_REPORTING_ERROR_REPORTING_CUSTOM_ERROR_HANDLER_HPP
