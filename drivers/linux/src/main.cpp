#include "emucad_usb/bridge.hpp"
#include "emucad_usb/options.hpp"

#include <iostream>

int main(int argc, char** argv) {
    const auto parsed = emucad_usb::parse_options(argc, argv);
    if (parsed.result == emucad_usb::ParseResult::Help) {
        std::cout << emucad_usb::usage();
        return 0;
    }
    if (parsed.result == emucad_usb::ParseResult::Version) {
        std::cout << emucad_usb::version() << "\n";
        return 0;
    }
    if (parsed.result == emucad_usb::ParseResult::Error) {
        std::cerr << parsed.error << "\n";
        return 2;
    }
    emucad_usb::Bridge bridge(parsed.options);
    return bridge.run();
}
