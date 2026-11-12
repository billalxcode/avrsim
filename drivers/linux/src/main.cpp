#include "embed_emu_usb/bridge.hpp"
#include "embed_emu_usb/options.hpp"

#include <iostream>

int main(int argc, char** argv) {
    const auto parsed = embed_emu_usb::parse_options(argc, argv);
    if (parsed.result == embed_emu_usb::ParseResult::Help) {
        std::cout << embed_emu_usb::usage();
        return 0;
    }
    if (parsed.result == embed_emu_usb::ParseResult::Version) {
        std::cout << embed_emu_usb::version() << "\n";
        return 0;
    }
    if (parsed.result == embed_emu_usb::ParseResult::Error) {
        std::cerr << parsed.error << "\n";
        return 2;
    }
    embed_emu_usb::Bridge bridge(parsed.options);
    return bridge.run();
}
