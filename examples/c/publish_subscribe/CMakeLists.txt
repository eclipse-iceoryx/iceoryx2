cmake_minimum_required(VERSION 3.28)
project(example_c_publish_subscribe)

find_package(iceoryx2_lang_c REQUIRED)

add_executable(example_c_publisher src/publisher.c)
target_link_libraries(example_c_publisher iceoryx2_lang_c)

add_executable(example_c_subscriber src/subscriber.c)
target_link_libraries(example_c_subscriber iceoryx2_lang_c)
