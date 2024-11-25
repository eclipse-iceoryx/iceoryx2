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

#include "iox2/file_descriptor.hpp"

namespace iox2 {
FileDescriptorView::FileDescriptorView(iox2_file_descriptor_ptr handle)
    : m_handle { handle } {
}

auto FileDescriptorView::file_descriptor() const -> FileDescriptorView {
    return *this;
}

auto FileDescriptorView::unsafe_native_handle() const -> int32_t {
    return iox2_file_descriptor_native_handle(m_handle);
}

auto FileDescriptor::create_owning(int32_t file_descriptor) -> iox::optional<FileDescriptor> {
    iox2_file_descriptor_h handle = nullptr;
    if (iox2_file_descriptor_new(file_descriptor, true, nullptr, &handle)) {
        return { FileDescriptor(handle) };
    }

    return iox::nullopt;
}

auto FileDescriptor::create_non_owning(int32_t file_descriptor) -> iox::optional<FileDescriptor> {
    iox2_file_descriptor_h handle = nullptr;
    if (iox2_file_descriptor_new(file_descriptor, false, nullptr, &handle)) {
        return { FileDescriptor(handle) };
    }

    return iox::nullopt;
}

FileDescriptor::FileDescriptor(iox2_file_descriptor_h handle)
    : m_handle { handle } {
}

FileDescriptor::FileDescriptor(FileDescriptor&& rhs) noexcept {
    *this = std::move(rhs);
}

auto FileDescriptor::operator=(FileDescriptor&& rhs) noexcept -> FileDescriptor& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

FileDescriptor::~FileDescriptor() {
    drop();
}

void FileDescriptor::drop() {
    if (m_handle != nullptr) {
        iox2_file_descriptor_drop(m_handle);
        m_handle = nullptr;
    }
}

auto FileDescriptor::unsafe_native_handle() const -> int32_t {
    return iox2_file_descriptor_native_handle(iox2_cast_file_descriptor_ptr(m_handle));
}

auto FileDescriptor::as_view() const -> FileDescriptorView {
    return FileDescriptorView(iox2_cast_file_descriptor_ptr(m_handle));
}


} // namespace iox2
