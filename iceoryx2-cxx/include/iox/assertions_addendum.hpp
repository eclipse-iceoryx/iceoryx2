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

#ifndef IOX2_ASSERTIONS_ADDENDUM_HPP
#define IOX2_ASSERTIONS_ADDENDUM_HPP

#include "iox2/legacy/assertions.hpp"

// NOLINTBEGIN(cppcoreguidelines-macro-usage)
#define IOX2_TODO() iox2::legacy::er::forwardPanic(IOX2_CURRENT_SOURCE_LOCATION, "Not yet implemented!")
// NOLINTEND(cppcoreguidelines-macro-usage)

#endif
