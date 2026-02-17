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

#include "iox2/type_variant.hpp"

auto operator<<(std::ostream& stream, const iox2::TypeVariant& value) -> std::ostream& {
    stream << "TypeVariant::";

    switch (value) {
    case iox2::TypeVariant::FixedSize: {
        stream << "Fixed";
        break;
    }
    case iox2::TypeVariant::Dynamic: {
        stream << "Dynamic";
        break;
    }
    }

    return stream;
}
