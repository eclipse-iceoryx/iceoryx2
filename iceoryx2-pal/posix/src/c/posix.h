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

#ifndef IOX2_PAL_POSIX_H
#define IOX2_PAL_POSIX_H

#ifdef __FreeBSD__
#include <libutil.h>
#include <sys/param.h>
#include <sys/sysctl.h>
#include <sys/ucred.h>
#include <sys/user.h>
#endif

#ifdef __APPLE__
#include <libproc.h>
#include <mach-o/dyld.h>
#endif

#ifdef __QNXNTO__
#include <sys/neutrino.h>
#include <sys/syspage.h> // needed for affinity setting
#endif

#if !(defined(_WIN64) || defined(_WIN32))
#include <arpa/inet.h>
#include <dirent.h>
#include <grp.h>
#include <netinet/in.h>
#include <pthread.h>
#include <pwd.h>
#include <sched.h>
#include <semaphore.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <unistd.h>
#endif

#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>

#endif // IOX2_PAL_POSIX_H
