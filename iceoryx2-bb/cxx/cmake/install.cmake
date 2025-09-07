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
set(TARGETS_EXPORT_NAME "${PROJECT_NAME}Targets" )
set(PROJECT_NAMESPACE "iceoryx2-bb-cxx" )

set(DESTINATION_BINDIR ${CMAKE_INSTALL_BINDIR})
set(DESTINATION_LIBDIR ${CMAKE_INSTALL_LIBDIR})
set(DESTINATION_INCLUDEDIR ${CMAKE_INSTALL_INCLUDEDIR}/${PREFIX})
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

#
########## export library ##########
#

# target directories
install(
    TARGETS iceoryx2-bb-containers-cxx
    EXPORT ${TARGETS_EXPORT_NAME}
    RUNTIME DESTINATION ${DESTINATION_BINDIR} COMPONENT bin
    LIBRARY DESTINATION ${DESTINATION_LIBDIR} COMPONENT lib
    ARCHIVE DESTINATION ${DESTINATION_LIBDIR} COMPONENT lib
)

# header
install(
    # the '/' at the end is important in order to not have the 'include' folder installed but only the content
    DIRECTORY ${PROJECT_SOURCE_DIR}/include/
    DESTINATION ${DESTINATION_INCLUDEDIR}
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

# package export
install(
    EXPORT ${TARGETS_EXPORT_NAME}
    NAMESPACE ${PROJECT_NAMESPACE}::
    DESTINATION ${DESTINATION_CONFIGDIR}
)
