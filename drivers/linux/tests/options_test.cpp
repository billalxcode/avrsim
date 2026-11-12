#include "embed_emu_usb/options.hpp"

#include <iostream>

namespace {

int expect(bool condition, const char* message) {
    if (!condition) {
        std::cerr << "FAIL: " << message << "\n";
        return 1;
    }
    return 0;
}

int test_defaults() {
    char program[] = "embed-emu-usb";
    char* argv[] = {program, nullptr};
    const auto parsed = embed_emu_usb::parse_options(1, argv);
    int failures = expect(parsed.result == embed_emu_usb::ParseResult::Ok, "defaults");
    failures += expect(parsed.options.socket_path == "/tmp/embed-emu-usb.sock", "socket");
    failures += expect(parsed.options.device_path == "/dev/ttyGS0", "device");
    return failures;
}

int test_paths() {
    char program[] = "embed-emu-usb";
    char socket_flag[] = "--socket";
    char socket_path[] = "/tmp/test.sock";
    char device_flag[] = "--device";
    char device_path[] = "/dev/ttyGS9";
    char* argv[] = {program, socket_flag, socket_path, device_flag, device_path, nullptr};
    const auto parsed = embed_emu_usb::parse_options(5, argv);
    int failures = expect(parsed.result == embed_emu_usb::ParseResult::Ok, "paths");
    failures += expect(parsed.options.socket_path == "/tmp/test.sock", "socket path");
    failures += expect(parsed.options.device_path == "/dev/ttyGS9", "device path");
    return failures;
}

int test_help_and_version() {
    char program[] = "embed-emu-usb";
    char help[] = "--help";
    char* help_argv[] = {program, help, nullptr};
    char version[] = "--version";
    char* version_argv[] = {program, version, nullptr};
    const auto help_parsed = embed_emu_usb::parse_options(2, help_argv);
    const auto version_parsed = embed_emu_usb::parse_options(2, version_argv);
    int failures = expect(help_parsed.result == embed_emu_usb::ParseResult::Help, "help");
    failures += expect(
        version_parsed.result == embed_emu_usb::ParseResult::Version,
        "version"
    );
    return failures;
}

int test_errors() {
    char program[] = "embed-emu-usb";
    char socket_flag[] = "--socket";
    char* missing_argv[] = {program, socket_flag, nullptr};
    char unknown_flag[] = "--unknown";
    char* unknown_argv[] = {program, unknown_flag, nullptr};
    const auto missing = embed_emu_usb::parse_options(2, missing_argv);
    const auto unknown = embed_emu_usb::parse_options(2, unknown_argv);
    int failures = expect(missing.result == embed_emu_usb::ParseResult::Error, "missing");
    failures += expect(
        unknown.result == embed_emu_usb::ParseResult::Error,
        "unknown"
    );
    return failures;
}

}  // namespace

int main() {
    int failures = 0;
    failures += test_defaults();
    failures += test_paths();
    failures += test_help_and_version();
    failures += test_errors();
    if (failures != 0) {
        std::cerr << failures << " failures\n";
        return 1;
    }
    std::cout << "options_test passed\n";
    return 0;
}
