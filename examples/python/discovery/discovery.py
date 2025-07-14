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

"""Service discovery example."""

import iceoryx2 as iox2

iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
services = iox2.Service.list(iox2.config.global_config(), iox2.ServiceType.Ipc)
for service in services:
    print(service)
