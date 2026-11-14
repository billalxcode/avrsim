#pragma once

#include <string>

namespace emucad_usb {

struct Options {
    bool help = false;
    bool version = false;
    std::string socket_path = "/tmp/emucad-usb.sock";
    std::string device_path = "/dev/ttyGS0";
};

enum class ParseResult {
    Ok,
    Help,
    Version,
    Error,
};

struct ParsedOptions {
    ParseResult result = ParseResult::Ok;
    Options options;
    std::string error;
};

ParsedOptions parse_options(int argc, char** argv);
std::string usage();
std::string version();

}  // namespace emucad_usb
