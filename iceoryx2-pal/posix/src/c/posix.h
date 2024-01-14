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

#ifdef __FreeBSD__
#include <mqueue.h>
#if defined(IOX2_ACL_SUPPORT) && !defined(IOX2_DOCS_RS_SUPPORT)
#include <sys/acl.h>
#endif
#include <sys/param.h>
#include <sys/sysctl.h>
#include <sys/ucred.h>
#include <sys/user.h>
#endif

#ifdef __linux__
#if defined(IOX2_ACL_SUPPORT) && !defined(IOX2_DOCS_RS_SUPPORT)
#include <acl/libacl.h>
#endif
#include <mqueue.h>
#endif

#ifndef _WIN64
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

#if defined(IOX2_DOCS_RS_SUPPORT) && defined(IOX2_ACL_SUPPORT)
///////////////////////////////
// stub libacl.h implementation
///////////////////////////////

typedef int acl_tag_t;
typedef unsigned int acl_perm_t;
typedef int acl_type_t;
typedef int acl_t;
typedef int acl_entry_t;
typedef int acl_permset_t;

#define ACL_EXECUTE 0x01
#define ACL_WRITE 0x02
#define ACL_READ 0x04

#define ACL_UNDEFINED_TAG 0
#define ACL_USER_OBJ 1
#define ACL_USER 2
#define ACL_GROUP_OBJ 3
#define ACL_GROUP 4
#define ACL_MASK 5
#define ACL_OTHER 6

#define ACL_FIRST_ENTRY 7
#define ACL_NEXT_ENTRY 8

int acl_get_perm(acl_permset_t, acl_perm_t) { return 0; }
acl_t acl_init(int) { return 0; }
int acl_free(void *) { return 0; }
int acl_valid(acl_t) { return 0; }
int acl_create_entry(acl_t *, acl_entry_t *) { return 0; }
int acl_get_entry(acl_t, int, acl_entry_t *) { return 0; }
int acl_add_perm(acl_permset_t, acl_perm_t) { return 0; }
int acl_clear_perms(acl_permset_t) { return 0; }
int acl_get_permset(acl_entry_t, acl_permset_t *) { return 0; }
int acl_set_permset(acl_entry_t, acl_permset_t) { return 0; }
void *acl_get_qualifier(acl_entry_t) { return NULL; }
int acl_set_qualifier(acl_entry_t, const void *) { return 0; }
int acl_get_tag_type(acl_entry_t, acl_tag_t *) { return 0; }
int acl_set_tag_type(acl_entry_t, acl_tag_t) { return 0; }
acl_t acl_get_fd(int) { return 0; }
int acl_set_fd(int, acl_t) { return 0; }
char *acl_to_text(acl_t, ssize_t *) { return NULL; }
acl_t acl_from_text(const char *) { return 0; }
#endif

#ifndef _WIN64
struct iox2_sigaction {
    size_t iox2_sa_handler;
    sigset_t iox2_sa_mask;
    int iox2_sa_flags;
};
#endif
