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

#include "iox2/messaging_pattern.hpp"

auto operator<<(std::ostream& stream, const iox2::MessagingPattern& value) -> std::ostream& {
    switch (value) {
    case iox2::MessagingPattern::PublishSubscribe:
        stream << "iox2::MessagingPattern::PublishSubscribe";
        break;
    case iox2::MessagingPattern::Event:
        stream << "iox2::MessagingPattern::Event";
        break;
    case iox2::MessagingPattern::RequestResponse:
        stream << "iox2::MessagingPattern::RequestResponse";
        break;
    }
    return stream;
}
