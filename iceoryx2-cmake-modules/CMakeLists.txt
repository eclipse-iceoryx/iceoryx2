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

cmake_minimum_required(VERSION 3.22)

set(IOX2_VERSION 0.7.0)
project(iceoryx2-cmake-modules VERSION ${IOX2_VERSION})

#
########## select correct platform ##########
#

if (CMAKE_SYSTEM_NAME MATCHES Windows)
    set(IOX2_INTERNAL_MODULE_PLATFORM_ID win CACHE INTERNAL "")
elseif (CMAKE_SYSTEM_NAME MATCHES Darwin)
    set(IOX2_INTERNAL_MODULE_PLATFORM_ID mac CACHE INTERNAL "")
else()
    set(IOX2_INTERNAL_MODULE_PLATFORM_ID generic CACHE INTERNAL "")
endif()

#
########## find_package in source tree ##########
#

set(${PROJECT_NAME}_DIR ${PROJECT_SOURCE_DIR}/cmake
    CACHE FILEPATH
    "${PROJECT_NAME}Config.cmake to make find_package(${PROJECT_NAME}) work in source tree!"
    FORCE
)

#
########## set variables for export ##########
#

include(GNUInstallDirs)

# set variables for library export
set(PACKAGE_VERSION_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}ConfigVersion.cmake" )
set(PACKAGE_CONFIG_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake" )

set(DESTINATION_CONFIGDIR ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME})
set(DESTINATION_DATAROOTDIR ${CMAKE_INSTALL_DATAROOTDIR})

# create package files
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    ${PACKAGE_VERSION_FILE}
    VERSION ${IOX2_VERSION}
    COMPATIBILITY AnyNewerVersion
)
configure_package_config_file(
    "cmake/Config.cmake.in"
    ${PACKAGE_CONFIG_FILE}
    INSTALL_DESTINATION ${DESTINATION_CONFIGDIR}
)

# module files
install(
    DIRECTORY ${PROJECT_SOURCE_DIR}/modules/
    DESTINATION ${DESTINATION_CONFIGDIR}/modules
    COMPONENT dev
)
install(
    DIRECTORY "${PROJECT_SOURCE_DIR}/platform/${IOX2_INTERNAL_MODULE_PLATFORM_ID}/modules/"
    DESTINATION ${DESTINATION_CONFIGDIR}/modules
    COMPONENT dev
)

# license
install(
    FILES ${PROJECT_SOURCE_DIR}/LICENSE-APACHE  ${PROJECT_SOURCE_DIR}/LICENSE-MIT
    DESTINATION ${DESTINATION_DATAROOTDIR}/doc/${PROJECT_NAME}
    COMPONENT dev
)

# package files
install(
    FILES ${PACKAGE_VERSION_FILE} ${PACKAGE_CONFIG_FILE}
    DESTINATION ${DESTINATION_CONFIGDIR}
    COMPONENT dev
)
