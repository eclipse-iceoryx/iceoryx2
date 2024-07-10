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
#ifndef IOX2_ATTRIBUTE_SET_HPP_
#define IOX2_ATTRIBUTE_SET_HPP_

#include <iostream>
#include <string>
#include <vector>

namespace iox2 {
class AttributeSet {
   public:
    std::vector<std::string> get(const std::string& key) const {}
};

inline std::ostream& operator<<(std::ostream& stream,
                                const AttributeSet& value) {
    std::cout << "AttributeSet { }";
    return stream;
}

}  // namespace iox2

#endif
