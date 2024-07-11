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

#if defined(_WIN64) || defined(_WIN32)
// clang-format off
// the include order is important, since some headers are defining macros that
// are used in the next header
#include <WinSock2.h>
#include <Windows.h>
#include <MSWSock.h>
#include <io.h>
// clang-format on
#else
#include <sys/select.h>
#include <sys/socket.h>

size_t iceoryx2_cmsg_space(const size_t len) {
    return CMSG_SPACE(len);
}

struct cmsghdr* iceoryx2_cmsg_firsthdr(const struct msghdr* hdr) {
    return CMSG_FIRSTHDR(hdr);
}

struct cmsghdr* iceoryx2_cmsg_nxthdr(struct msghdr* hdr, struct cmsghdr* sub) {
    return CMSG_NXTHDR(hdr, sub);
}

size_t iceoryx2_cmsg_len(const size_t len) {
    return CMSG_LEN(len);
}

unsigned char* iceoryx2_cmsg_data(struct cmsghdr* cmsg) {
    return CMSG_DATA(cmsg);
}

void iceoryx2_fd_clr(const int fd, fd_set* set) {
    FD_CLR(fd, set);
}

int iceoryx2_fd_isset(const int fd, const fd_set* set) {
    return FD_ISSET(fd, set);
}

void iceoryx2_fd_set(const int fd, fd_set* set) {
    FD_SET(fd, set);
}

void iceoryx2_fd_zero(fd_set* set) {
    FD_ZERO(set);
}

#endif
