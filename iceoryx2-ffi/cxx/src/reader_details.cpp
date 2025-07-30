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

#include "iox2/reader_details.hpp"

namespace iox2 {
ReaderDetailsView::ReaderDetailsView(/*iox2_reader_details_ptr handle*/) {
    IOX_TODO();
}

ReaderDetailsView::ReaderDetailsView([[maybe_unused]] ReaderDetailsView&& rhs) noexcept {
    IOX_TODO();
}

auto ReaderDetailsView::operator=([[maybe_unused]] ReaderDetailsView&& rhs) noexcept -> ReaderDetailsView& {
    IOX_TODO();
}

auto ReaderDetailsView::reader_id() const -> UniqueReaderId {
    IOX_TODO();
}

auto ReaderDetailsView::node_id() const -> NodeId {
    IOX_TODO();
}
} // namespace iox2

