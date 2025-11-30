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

#ifndef IOX2_BB_UTILITY_STD_CHRONO_SUPPORT_INL
#define IOX2_BB_UTILITY_STD_CHRONO_SUPPORT_INL

#include "iox2/legacy/std_chrono_support.hpp"

namespace iox2 {
namespace bb {

inline legacy::units::Duration
FromTrait<std::chrono::nanoseconds, legacy::units::Duration>::from(const std::chrono::nanoseconds& value) noexcept {
    return legacy::units::Duration::fromNanoseconds(value.count());
}
inline legacy::units::Duration FromTrait<std::chrono::microseconds, legacy::units::Duration>::from(
    const std::chrono::microseconds& value) noexcept {
    return legacy::units::Duration::fromMicroseconds(value.count());
}

inline legacy::units::Duration FromTrait<std::chrono::milliseconds, legacy::units::Duration>::from(
    const std::chrono::milliseconds& value) noexcept {
    return legacy::units::Duration::fromMilliseconds(value.count());
}

inline legacy::units::Duration
FromTrait<std::chrono::seconds, legacy::units::Duration>::from(const std::chrono::seconds& value) noexcept {
    return legacy::units::Duration::fromSeconds(value.count());
}

} // namespace bb
} // namespace iox2

#endif // IOX2_BB_UTILITY_STD_CHRONO_SUPPORT_INL
