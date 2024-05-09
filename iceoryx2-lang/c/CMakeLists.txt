cmake_minimum_required(VERSION 3.28)
project(iceoryx2_lang_c)

# TODO remove once generation works as expected ... have a look at iceoryx-rs
configure_file("${CMAKE_CURRENT_SOURCE_DIR}/iceoryx2.h"
  "${CMAKE_BINARY_DIR}/generated/include/iox2/iceoryx2.h" @ONLY)

set(CMAKE_FIND_LIBRARY_SUFFIXES ".a")
list(APPEND CMAKE_FIND_LIBRARY_SUFFIXES ".so")
list(APPEND CMAKE_FIND_LIBRARY_SUFFIXES ".dll")

find_library(ICEORYX2_LANG_C_LIB ${PROJECT_NAME} HINTS ${iceoryx2_SOURCE_DIR}/target/release)

add_library(${PROJECT_NAME} INTERFACE)

target_include_directories(${PROJECT_NAME} INTERFACE ${CMAKE_BINARY_DIR}/generated/include)

target_link_libraries(${PROJECT_NAME} INTERFACE ${ICEORYX2_LANG_C_LIB})

########## find_package in source tree ##########
set(${PROJECT_NAME}_DIR ${PROJECT_SOURCE_DIR}/cmake
    CACHE FILEPATH
    "${PROJECT_NAME}Config.cmake to make find_package(${PROJECT_NAME}) work in source tree!"
    FORCE
)
