# Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#!/bin/bash

sudo apt-get update
sudo apt-get install -y clang cmake curl gcc g++ git libacl1-dev build-essential flex libelf-dev binutils-dev libdwarf-dev libc6-dev libc6-dev-i386 gcc-multilib-i686-linux-gnu libc6-dev-i386-cross
sudo useradd testuser1
sudo useradd testuser2
sudo groupadd testgroup1
sudo groupadd testgroup2
