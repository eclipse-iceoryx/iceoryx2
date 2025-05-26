#!/usr/bin/env bash
# Copyright (c) 2023 Contributors to the Eclipse Foundation
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

set -e

cd $(git rev-parse --show-toplevel)

set_license_header() {
    FILES=$(find . -type f -iwholename "${FILE_SUFFIX}" )

    for FILE in $FILES
    do
        sed -i "1s/^/\n/" $FILE
        sed -i "1s/^/$C SPDX-License-Identifier: Apache-2.0 OR MIT\n/" $FILE
        sed -i "1s/^/$C\n/" $FILE
        sed -i "1s/^/$C which is available at https:\/\/opensource.org\/licenses\/MIT.\n/" $FILE
        sed -i "1s/^/$C https:\/\/www.apache.org\/licenses\/LICENSE-2.0, or the MIT license\n/" $FILE
        sed -i "1s/^/$C terms of the Apache Software License 2.0 which is available at\n/" $FILE
        sed -i "1s/^/$C This program and the accompanying materials are made available under the\n/" $FILE
        sed -i "1s/^/$C\n/" $FILE
        sed -i "1s/^/$C information regarding copyright ownership.\n/" $FILE
        sed -i "1s/^/$C See the NOTICE file(s) distributed with this work for additional\n/" $FILE
        sed -i "1s/^/$C\n/" $FILE
        sed -i "1s/^/$C Copyright (c) 2023 Contributors to the Eclipse Foundation\n/" $FILE
    done
}

set_rust() {
    FILE_SUFFIX="*.rs"
    C="\/\/"
    set_license_header
}

set_shell() {
    FILE_SUFFIX="*.sh"
    C="#"
    set_license_header
}

set_toml() {
    FILE_SUFFIX="*.toml"
    C="#"
    set_license_header
}

set_rust
set_shell

# no toml check for now
# it is usually only some configuration files which can be used without copyright notice
# set_toml

