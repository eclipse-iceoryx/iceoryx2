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

#ifndef IOX2_INCLUDE_GUARD_COMPONENT_TESTS_COMMON_HPP
#define IOX2_INCLUDE_GUARD_COMPONENT_TESTS_COMMON_HPP

#include <iox2/iceoryx2.hpp>

#include <memory>

class IComponentTest {
protected:
    IComponentTest() = default;
public:
    virtual ~IComponentTest() = default;
    IComponentTest(IComponentTest const&) = delete;
    IComponentTest(IComponentTest&&) = delete;
    auto operator=(IComponentTest const&) -> IComponentTest& = delete;
    auto operator=(IComponentTest&&)  -> IComponentTest& = delete;

    virtual auto test_name() const -> char const* = 0;
    virtual auto run_test(iox2::Node<iox2::ServiceType::Ipc> const& node) -> bool = 0;
};

auto test_containers() -> std::unique_ptr<IComponentTest>;

#endif
