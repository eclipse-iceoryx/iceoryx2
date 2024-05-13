cmake_minimum_required(VERSION 3.28)
project(iceoryx2_lang_c)

########## Rust target ##########

set(RUST_BUILD_TYPE "release")
set(RUST_BUILD_TYPE_FLAG "--${RUST_BUILD_TYPE}")
if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(RUST_BUILD_TYPE "debug")
    set(RUST_BUILD_TYPE_FLAG "")
endif()

set(RUST_ARCH_TRIPLET "")
set(RUST_ARCH_TRIPLET_FLAG "")
# if(${CMAKE_SYSTEM_PROCESSOR} STREQUAL "i486")
#     set(RUST_ARCH_TRIPLET "i686-unknown-linux-gnu")
#     set(RUST_ARCH_TRIPLET_FLAG "--target=${RUST_ARCH_TRIPLET}")
# elseif(${CMAKE_SYSTEM_PROCESSOR} STREQUAL "armv7l")
#     set(RUST_ARCH_TRIPLET "armv7-unknown-linux-gnueabihf")
#     set(RUST_ARCH_TRIPLET_FLAG "--target=${RUST_ARCH_TRIPLET}")
# elseif(${CMAKE_SYSTEM_PROCESSOR} STREQUAL "aarch64")
#     set(RUST_ARCH_TRIPLET "${CMAKE_SYSTEM_PROCESSOR}-unknown-linux-gnu")
#     set(RUST_ARCH_TRIPLET_FLAG "--target=${RUST_ARCH_TRIPLET}")
# endif()

set(RUST_TARGET_DIR ${CMAKE_BINARY_DIR}/rust)
set(RUST_ARCH_TARGET_DIR ${RUST_TARGET_DIR}/${RUST_ARCH_TRIPLET})

set(ICEORYX2_LANG_C_LIB ${RUST_ARCH_TARGET_DIR}/${RUST_BUILD_TYPE}/libiceoryx2_lang_c.a)

# run cargo
add_custom_target(
    iceoryx2 ALL
    COMMAND cargo build ${RUST_BUILD_TYPE_FLAG} --target-dir=${RUST_TARGET_DIR} ${RUST_ARCH_TRIPLET_FLAG}
    BYPRODUCTS
    "${CMAKE_CURRENT_SOURCE_DIR}/iceoryx2.h"
    ${ICEORYX2_LANG_C_LIB}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    VERBATIM
    USES_TERMINAL
)

# TODO remove once generation works as expected ... have a look at iceoryx-rs
configure_file("${CMAKE_CURRENT_SOURCE_DIR}/iceoryx2.h"
  "${CMAKE_BINARY_DIR}/generated/include/iox2/iceoryx2.h" @ONLY)

# set(CMAKE_FIND_LIBRARY_SUFFIXES ".a")
# list(APPEND CMAKE_FIND_LIBRARY_SUFFIXES ".so")
# list(APPEND CMAKE_FIND_LIBRARY_SUFFIXES ".dll")
#
# find_library(ICEORYX2_LANG_C_LIB ${PROJECT_NAME} HINTS ${iceoryx2_SOURCE_DIR}/target/release)

########## C target ##########

add_library(${PROJECT_NAME} INTERFACE)

add_dependencies(${PROJECT_NAME} iceoryx2)

target_include_directories(${PROJECT_NAME} INTERFACE ${CMAKE_BINARY_DIR}/generated/include)

target_link_libraries(${PROJECT_NAME} INTERFACE ${ICEORYX2_LANG_C_LIB})

########## find_package in source tree ##########
set(${PROJECT_NAME}_DIR ${PROJECT_SOURCE_DIR}/cmake
    CACHE FILEPATH
    "${PROJECT_NAME}Config.cmake to make find_package(${PROJECT_NAME}) work in source tree!"
    FORCE
)
