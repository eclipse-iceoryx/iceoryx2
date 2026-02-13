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

#include "iox2/version.hpp"

#include <ostream>

namespace iox2 {

PackageVersion package_version() {
    iox2_package_version_t const v = iox2_package_version();
    return PackageVersion { v.major, v.minor, v.patch };
}

auto operator<<(std::ostream& stream, const PackageVersion& version) -> std::ostream& {
    return stream << version.major << '.' << version.minor << '.' << version.patch;
}

auto operator==(const PackageVersion& lhs, const PackageVersion& rhs) -> bool {
    return lhs.major == rhs.major && lhs.minor == rhs.minor && lhs.patch == rhs.patch;
}

auto operator<(const PackageVersion& lhs, const PackageVersion& rhs) -> bool {
    if (lhs.major != rhs.major) {
        return lhs.major < rhs.major;
    } else if (lhs.minor != rhs.minor) {
        return lhs.minor < rhs.minor;
    } else {
        return lhs.patch < rhs.patch;
    }
}

} // namespace iox2
