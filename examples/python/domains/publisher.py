# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

"""Publisher example."""

import ctypes
import sys

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)
iox2.set_log_level_from_env_or(iox2.LogLevel.Info)

domain = sys.argv[1]
service_name = sys.argv[2]

# create a new config based on the global config
config = iox2.config.global_config()

# The domain name becomes the prefix for all resources.
# Therefore, different domain names never share the same resources.
config.global_cfg.prefix = iox2.FileName.new(domain)

node = (
    iox2.NodeBuilder.new()
    # use the custom config when creating the custom node
    # every service constructed by the node will use this config
    .config(config).create(iox2.ServiceType.Ipc)
)

# from here on it is the publish_subscribe publisher example
service = (
    node.service_builder(iox2.ServiceName.new(service_name))
    .publish_subscribe(ctypes.c_uint64)
    .open_or_create()
)

publisher = service.publisher_builder().create()

COUNTER = 0
try:
    while True:
        COUNTER += 1
        node.wait(cycle_time)
        publisher.send_copy(ctypes.c_uint64(COUNTER))
        print(
            '[domain: "',
            domain,
            '", service: "',
            service_name,
            '"] Send sample',
            COUNTER,
            "...",
        )

except iox2.NodeWaitFailure:
    print("exit")
