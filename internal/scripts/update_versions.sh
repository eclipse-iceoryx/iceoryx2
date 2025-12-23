#!/usr/bin/env bash
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

set -e

C_OFF='\033[0m'
C_RED='\033[1;31m'
C_GREEN='\033[1;32m'
C_YELLOW='\033[1;33m'
C_BLUE='\033[1;34m'

UPDATE_ICEORYX2_VERSION=false

while (( "$#" )); do
    case "$1" in
        --iceoryx2)
            NEW_ICEORYX2_VERSION=$2
            UPDATE_ICEORYX2_VERSION=true
            shift 2
            ;;
        "help")
            echo -e "Script to update the iceoryx2 version"
            echo -e ""
            echo -e ""
            echo -e "Usage: ${C_GREEN}$(basename $0)${C_OFF} ${C_BLUE}SCRIPT-OPTION${C_OFF}"
            echo -e "Command:"
            echo -e "    help                          Print this help"
            echo -e "Options:"
            echo -e "    "
            echo -e "    --iceoryx2 <VERSION>          Change all iceoryx2 versions to <VERSION>"
            echo -e ""
            exit 0
            ;;
        *)
            echo -e "${C_RED}ERROR:${C_OFF} Invalid argument '$1'. Try 'help' for options."
            exit 1
            ;;
    esac
done

if [[ ${UPDATE_ICEORYX2_VERSION} == false ]]; then
    echo -e "${C_RED}ERROR:${C_OFF} No arguments provided. Try 'help' for options."
    exit 1
fi

cd $(git rev-parse --show-toplevel)

update_package_version_rs() {
    OLD=$1
    NEW=$2

    IFS='.' read -r OLD_MAJOR OLD_MINOR OLD_PATCH <<< ${OLD}
    IFS='.' read -r NEW_MAJOR NEW_MINOR NEW_PATCH <<< ${NEW}

    sed -i 's/const MAJOR: u16 = '"${OLD_MAJOR}"';/const MAJOR: u16 = '"${NEW_MAJOR}"';/g' \
    iceoryx2-bb/elementary/src/package_version.rs
    sed -i 's/const MINOR: u16 = '"${OLD_MINOR}"';/const MINOR: u16 = '"${NEW_MINOR}"';/g' \
    iceoryx2-bb/elementary/src/package_version.rs
    sed -i 's/const PATCH: u16 = '"${OLD_PATCH}"';/const PATCH: u16 = '"${NEW_PATCH}"';/g' \
    iceoryx2-bb/elementary/src/package_version.rs
}

if [[ ${UPDATE_ICEORYX2_VERSION} == true ]]; then
    echo -e "Updating ${C_BLUE}iceoryx2${C_OFF} version to: ${C_BLUE}${NEW_ICEORYX2_VERSION}${C_OFF}!"

    OLD_VERSION=$(grep 'CURRENT:' internal/VERSIONS | sed 's/CURRENT: //')
    NEW_VERSION=${NEW_ICEORYX2_VERSION}

    update_package_version_rs ${OLD_VERSION} ${NEW_VERSION}

    sed -i 's/^version = "'"${OLD_VERSION}"'"/version = "'"${NEW_VERSION}"'"/g' \
        Cargo.toml
    sed -i 's/^version = "'"${OLD_VERSION}"'"/version = "'"${NEW_VERSION}"'"/g' \
        examples/nostd/posix/rust/Cargo.toml

    find . -name "Cargo.toml" -type f -exec \
        sed -i 's/"'"${OLD_VERSION}"'", path = "iceoryx2/"'"${NEW_VERSION}"'", path = "iceoryx2/g' {} \;

    find . -name "CMakeLists.txt" -type f -exec \
        sed -i 's/set(IOX2_VERSION '"${OLD_VERSION}"')/set\(IOX2_VERSION '"${NEW_VERSION}"')/g' {} \;

    find . -name "*.cmake" -type f -exec \
        sed -i 's/set(IOX2_VERSION '"${OLD_VERSION}"')/set\(IOX2_VERSION '"${NEW_VERSION}"')/g' {} \;

    find . -name "CMakeLists.txt" -type f -exec \
        sed -i 's/find_package(iceoryx2-c '"${OLD_VERSION}"'/find_package(iceoryx2-c '"${NEW_VERSION}"'/g' {} \;
    find . -name "CMakeLists.txt" -type f -exec \
        sed -i 's/find_package(iceoryx2-cxx '"${OLD_VERSION}"'/find_package(iceoryx2-cxx '"${NEW_VERSION}"'/g' {} \;

    find . -name "pyproject.toml" -type f -exec \
        sed -i 's/version = "'"${OLD_VERSION}"'"/version = "'"${NEW_VERSION}"'"/g' {} \;

    sed -i -z 's/name = "iceoryx2",\n    version = "'"${OLD_VERSION}"'"/name = "iceoryx2",\n    version = "'"${NEW_VERSION}"'"/g' \
        MODULE.bazel

    sed -i 's/    <version>'"${OLD_VERSION}"'/    <version>'"${NEW_VERSION}"'/g' \
        package.xml

    sed -i 's/pip install iceoryx2=='"${OLD_VERSION}"'/pip install iceoryx2=='"${NEW_VERSION}"'/g' \
        examples/python/README.md
    sed -i 's/git checkout v'"${OLD_VERSION}"'/git checkout v'"${NEW_VERSION}"'/g' \
        examples/python/README.md

    sed -i 's/iceoryx2\/v'"${OLD_VERSION}"'/iceoryx2\/v'"${NEW_VERSION}"'/g' \
        doc/user-documentation/use-iceoryx2-with-zig.md

    if grep -rF \
        --exclude-dir=.env \
        --exclude-dir=.git \
        --exclude-dir=landing-page \
        --exclude-dir=plots \
        --exclude-dir=release-notes \
        --exclude-dir=target \
        --exclude=Cargo.lock \
        --exclude=Cargo.Bazel.lock \
        --exclude=MODULE.bazel.lock \
        --exclude=CHANGELOG.md \
        --exclude=header.html \
        --exclude=poetry.lock \
        --exclude=VERSIONS \
        ${OLD_VERSION} | grep iceoryx2; then

        echo -e "${C_RED}ERROR:${C_OFF} Found the old iceoryx2 version string!"
        echo -e "Please update the script to include the new occurrences of '${OLD_VERSION}'"

        exit 1
    fi

    sed -i 's/CURRENT: '"${OLD_VERSION}"'/CURRENT: '"${NEW_VERSION}"'/g' \
        internal/VERSIONS

    if grep -q "CURRENT: ${OLD_VERSION}" internal/VERSIONS; then
        echo -e "${C_RED}ERROR:${C_OFF} Could not update 'CURRENT' version in 'internal/VERSIONS'"

        exit 1
    fi

    echo -e "${C_GREEN}Successuflly updated the iceoryx2 version to '${NEW_VERSION}'${C_OFF}!"
    echo -e "${C_YELLOW}Please also build iceoryx2 with cargo and bazel and the python bindings to update the lock files!${C_OFF}"

fi
