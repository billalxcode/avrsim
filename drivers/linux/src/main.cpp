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
    std::cout << "embed-emu-usb scaffold\n";
    std::cout << "socket=" << parsed.options.socket_path << "\n";
    std::cout << "device=" << parsed.options.device_path << "\n";
    std::cout << "USB transport not implemented yet\n";
    return 0;
}
