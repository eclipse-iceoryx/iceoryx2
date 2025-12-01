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

#include "iox2/bb/path.hpp"
#include "iox2/legacy/attributes.hpp"

namespace iox2 {
namespace legacy {
namespace detail {
auto path_does_contain_invalid_content(const string<platform::IOX2_MAX_PATH_LENGTH>& value IOX2_MAYBE_UNUSED) noexcept
    -> bool {
    return false;
}
} // namespace detail
} // namespace legacy
} // namespace iox2
