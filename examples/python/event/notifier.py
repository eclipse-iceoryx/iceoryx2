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

"""Notifier example for python."""

import iceoryx2_ffi_python as iceoryx2

config = iceoryx2.config.default()

node_name = iceoryx2.NodeName.new("fuubar")
node = iceoryx2.NodeBuilder.new().name(node_name).config(config).create(iceoryx2.ServiceType.Ipc)
cycle_time = iceoryx2.Duration.from_millis(250)

for count in range(100):
    print("fuu")
    node.wait(cycle_time)
