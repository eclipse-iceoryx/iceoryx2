cmake_minimum_required(VERSION 3.28)
project(example_c_discovery)

find_package(iceoryx2_lang_c REQUIRED)

add_executable(example_c_discovery src/main.c)

target_link_libraries(example_c_discovery iceoryx2_lang_c)
