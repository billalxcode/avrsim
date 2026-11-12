cmake_minimum_required(VERSION 3.20)

project(embed_emu_usb VERSION 0.1.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_library(embed_emu_usb_core STATIC
    src/bridge.cpp
    src/options.cpp
)

target_include_directories(embed_emu_usb_core PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

target_compile_options(embed_emu_usb_core PRIVATE
    -Wall
    -Wextra
    -Wpedantic
)

add_executable(embed-emu-usb
    src/main.cpp
)

target_link_libraries(embed-emu-usb PRIVATE embed_emu_usb_core)

target_compile_options(embed-emu-usb PRIVATE
    -Wall
    -Wextra
    -Wpedantic
)

install(TARGETS embed-emu-usb RUNTIME DESTINATION bin)

include(CTest)

if(BUILD_TESTING)
    add_executable(options_test
        tests/options_test.cpp
    )
    target_link_libraries(options_test PRIVATE embed_emu_usb_core)
    target_compile_options(options_test PRIVATE
        -Wall
        -Wextra
        -Wpedantic
    )
    add_test(NAME options_test COMMAND options_test)
endif()
