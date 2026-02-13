// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#ifndef IOX2_VERSION_HPP
#define IOX2_VERSION_HPP

#include "iox2/internal/iceoryx2.hpp"

#include <cstdint>
#include <iosfwd>

namespace iox2 {

/// Version number.
struct PackageVersion {
    std::uint16_t major;
    std::uint16_t minor;
    std::uint16_t patch;
};

/// Returns the crates version acquired through the internal environment variables set by cargo,
/// ("CARGO_PKG_VERSION_{MAJOR|MINOR|PATCH}").
PackageVersion package_version();

auto operator<<(std::ostream& stream, const PackageVersion& version) -> std::ostream&;
auto operator==(const PackageVersion& lhs, const PackageVersion& rhs) -> bool;
auto operator<(const PackageVersion& lhs, const PackageVersion& rhs) -> bool;

} // namespace iox2

#endif
