#include "embed_emu_usb/options.hpp"

namespace embed_emu_usb {
namespace {

constexpr const char* kVersion = "0.1.0";

bool fail(ParsedOptions& parsed, const std::string& message) {
    parsed.result = ParseResult::Error;
    parsed.error = message;
    return false;
}

void assign_path(Options& options, const std::string& name, const char* value) {
    if (name == "--socket") {
        options.socket_path = value;
        return;
    }
    options.device_path = value;
}

bool parse_path(
    ParsedOptions& parsed,
    int argc,
    char** argv,
    int& index,
    const std::string& name
) {
    if (index + 1 >= argc) {
        return fail(parsed, "missing value for " + name);
    }
    assign_path(parsed.options, name, argv[++index]);
    return true;
}

bool parse_one(ParsedOptions& parsed, int argc, char** argv, int& index) {
    const std::string argument = argv[index];
    if (argument == "--help" || argument == "-h") {
        parsed.result = ParseResult::Help;
        return false;
    }
    if (argument == "--version") {
        parsed.result = ParseResult::Version;
        return false;
    }
    if (argument == "--socket" || argument == "--device") {
        return parse_path(parsed, argc, argv, index, argument);
    }
    return fail(parsed, "unknown option: " + argument);
}

}  // namespace

ParsedOptions parse_options(int argc, char** argv) {
    ParsedOptions parsed;
    for (int index = 1; index < argc; ++index) {
        if (!parse_one(parsed, argc, argv, index)) {
            return parsed;
        }
    }
    return parsed;
}

std::string usage() {
    return "usage: embed-emu-usb [--socket PATH] [--device PATH]\n"
           "\n"
           "Options:\n"
           "  --socket PATH  Unix socket for the simulator\n"
           "  --device PATH  Gadget serial device\n"
           "  --help         Show this help\n"
           "  --version      Show version\n";
}

std::string version() {
    return std::string("embed-emu-usb ") + kVersion;
}

}  // namespace embed_emu_usb
