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

# NOTE the file is included in '../CMakeLists.txt' and therefore all paths based on 'CMAKE_CURRENT_SOURCE_DIR' must be relative to '../'

#
########## find_package in source tree ##########
#

set(${PROJECT_NAME}_DIR ${PROJECT_SOURCE_DIR}/cmake
    CACHE FILEPATH
    "${PROJECT_NAME}Config.cmake to make find_package(${PROJECT_NAME}) work in source tree!"
    FORCE
)

if(ICEORYX_WITH_FETCH_CONTENT)
    return()
endif()

#
########## set variables for export ##########
#

include(GNUInstallDirs)

# set variables for library export
set(PACKAGE_VERSION_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}ConfigVersion.cmake" )
set(PACKAGE_CONFIG_FILE "${CMAKE_CURRENT_BINARY_DIR}/${PROJECT_NAME}Config.cmake" )
set(TARGETS_EXPORT_NAME "${PROJECT_NAME}Targets" )
set(PROJECT_NAMESPACE "iceoryx2-c" )

set(DESTINATION_BINDIR ${CMAKE_INSTALL_BINDIR})
set(DESTINATION_LIBDIR ${CMAKE_INSTALL_LIBDIR})
set(DESTINATION_INCLUDEDIR ${CMAKE_INSTALL_INCLUDEDIR}/${PREFIX})
set(DESTINATION_CONFIGDIR ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME})
set(DESTINATION_DATAROOTDIR ${CMAKE_INSTALL_DATAROOTDIR})

# create package files
include(CMakePackageConfigHelpers)
write_basic_package_version_file(
    ${PACKAGE_VERSION_FILE}
    VERSION ${IOX2_VERSION_STRING}
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
if(BUILD_SHARED_LIBS)
install(
    TARGETS includes-only static-lib shared-lib
    EXPORT ${TARGETS_EXPORT_NAME}
    RUNTIME DESTINATION ${DESTINATION_BINDIR} COMPONENT bin
    LIBRARY DESTINATION ${DESTINATION_LIBDIR} COMPONENT lib
    ARCHIVE DESTINATION ${DESTINATION_LIBDIR} COMPONENT lib
)
else()
install(
    TARGETS includes-only static-lib
    EXPORT ${TARGETS_EXPORT_NAME}
    RUNTIME DESTINATION ${DESTINATION_BINDIR} COMPONENT bin
    LIBRARY DESTINATION ${DESTINATION_LIBDIR} COMPONENT lib
    ARCHIVE DESTINATION ${DESTINATION_LIBDIR} COMPONENT lib
)
endif()

# header
install(
    # the '/' at the end is important in order to not have the 'include' folder installed but only the content
    DIRECTORY ${ICEORYX2_C_INCLUDE_DIR}/
    DESTINATION ${DESTINATION_INCLUDEDIR}
    COMPONENT dev
)

# lib
install(
    FILES ${ICEORYX2_C_LIB_ARTIFACTS}
    DESTINATION ${DESTINATION_LIBDIR}
    COMPONENT lib
)

# license
install(
    FILES ${CMAKE_CURRENT_SOURCE_DIR}/../../LICENSE-APACHE  ${CMAKE_CURRENT_SOURCE_DIR}/../../LICENSE-MIT
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
