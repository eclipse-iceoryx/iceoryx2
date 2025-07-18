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

"""Discovery example."""

import sys

import iceoryx2 as iox2

cycle_time = iox2.Duration.from_secs(1)
iox2.set_log_level_from_env_or(iox2.LogLevel.Info)

domain = sys.argv[1]

# create a new config based on the global config
config = iox2.config.global_config()

# The domain name becomes the prefix for all resources.
# Therefore, different domain names never share the same resources.
config.global_cfg.prefix = iox2.FileName.new(domain)

print(f'Services running in domain "{domain}":')

# use the custom config when listing the services
services = iox2.Service.list(config, iox2.ServiceType.Ipc)
for service in services:
    print(service)
