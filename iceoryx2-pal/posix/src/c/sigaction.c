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

#if !(defined(_WIN64) || defined(_WIN32))
#include "posix.h"

int iox2_sigaction_func(int sig, const struct iox2_sigaction* restrict act, struct iox2_sigaction* restrict oact) {
    struct sigaction tr_act;
    // NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling) safe due to use of 'sizeof'
    memset(&tr_act, 0, sizeof(struct sigaction));
    struct sigaction* tr_act_ptr = NULL;

    struct sigaction tr_oact;
    // NOLINTNEXTLINE(clang-analyzer-security.insecureAPI.DeprecatedOrUnsafeBufferHandling) safe due to use of 'sizeof'
    memset(&tr_act, 0, sizeof(struct sigaction));
    struct sigaction* tr_oact_ptr = NULL;

    if (act != NULL) {
        tr_act.sa_flags = act->iox2_sa_flags;
        tr_act.sa_mask = act->iox2_sa_mask;
        // NOLINTNEXTLINE(performance-no-int-to-ptr) not an issue here since this is not in a hop path
        tr_act.sa_handler = (void (*)(int)) act->iox2_sa_handler;
        tr_act_ptr = &tr_act;
    }

    if (oact != NULL) {
        tr_oact_ptr = &tr_oact;
    }

    int ret_val = sigaction(sig, tr_act_ptr, tr_oact_ptr);

    if (ret_val == 0 && oact != NULL) {
        oact->iox2_sa_flags = tr_oact.sa_flags;
        oact->iox2_sa_mask = tr_oact.sa_mask;
        oact->iox2_sa_handler = (size_t) tr_oact.sa_handler;
    }

    return ret_val;
}
#endif
