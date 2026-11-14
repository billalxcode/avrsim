cmake_minimum_required(VERSION 3.20)

project(emucad_usb VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(emucad_usb_core STATIC
    src/bridge.cpp
    src/options.cpp
)

target_include_directories(emucad_usb_core PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_compile_options(emucad_usb_core PRIVATE
    -Wall
    -Wextra
    -Wpedantic
)

add_executable(emucad-usb
    src/main.cpp
)

target_link_libraries(emucad-usb PRIVATE emucad_usb_core)

target_compile_options(emucad-usb PRIVATE
    -Wall
    -Wextra
    -Wpedantic
)

install(TARGETS emucad-usb RUNTIME DESTINATION bin)

include(CTest)

if(BUILD_TESTING)
    add_executable(options_test
        tests/options_test.cpp
    )
    target_link_libraries(options_test PRIVATE emucad_usb_core)
    target_compile_options(options_test PRIVATE
        -Wall
        -Wextra
        -Wpedantic
    )
    add_test(NAME options_test COMMAND options_test)
endif()
