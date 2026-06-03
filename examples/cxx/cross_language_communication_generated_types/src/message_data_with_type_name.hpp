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

#ifndef IOX2_EXAMPLES_MESSAGE_DATA_WITH_TYPE_NAME_HPP
#define IOX2_EXAMPLES_MESSAGE_DATA_WITH_TYPE_NAME_HPP

#include "iox2/type_name.hpp"
#include "message_data.hpp"

// This header makes the (generated or unmodifiable) types from `message_data.hpp`
// cross-language-compatible by attaching their cross-language type names with
// `IOX2_DEFINE_TYPE_NAME`.
// It is the non-intrusive alternative to the `IOX2_TYPE_NAME` member and is meant
// for types you do not own - e.g. structs emitted by an IDL/code generator.
IOX2_DEFINE_TYPE_NAME(TransmissionData, "TransmissionData");
IOX2_DEFINE_TYPE_NAME(CustomHeader, "CustomHeader");

#endif
