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

// NOTE: dummy source file to create static and shared libraries; can be removed
// once we have real cpp files or the libraries need to become an INTERFACE
// library in cmake

#include <iostream>
#include <iox/logging.hpp>

#include "iox2/iceoryx2.h"

void hypnotoad() {
    IOX_LOG(INFO, "All glory to the hypnotoad!");
    iox2_node_builder_new(nullptr);
}
