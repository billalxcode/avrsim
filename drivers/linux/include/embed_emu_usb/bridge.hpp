#pragma once

#include "embed_emu_usb/options.hpp"

#include <poll.h>

namespace embed_emu_usb {

class Bridge {
public:
    explicit Bridge(Options options);

    int run();

private:
    void serve(int server);
    void handle_client(int client);
    void bridge_client(int client, int device);
    bool bridge_ready(pollfd& client, pollfd& device, int client_fd, int device_fd);
    bool forward(int from, int to) const;

    Options options_;
};

}  // namespace embed_emu_usb
