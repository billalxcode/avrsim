#include "embed_emu_usb/bridge.hpp"

#include <cerrno>
#include <csignal>
#include <cstdio>
#include <cstring>
#include <fcntl.h>
#include <iostream>
#include <poll.h>
#include <signal.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/un.h>
#include <termios.h>
#include <unistd.h>
#include <utility>

namespace embed_emu_usb {
namespace {

volatile std::sig_atomic_t g_stop = 0;

void handle_signal(int) {
    g_stop = 1;
}

void install_signals() {
    struct sigaction action {};
    action.sa_handler = handle_signal;
    sigemptyset(&action.sa_mask);
    action.sa_flags = 0;
    sigaction(SIGINT, &action, nullptr);
    sigaction(SIGTERM, &action, nullptr);
}

int fail(const char* what) {
    std::perror(what);
    return 1;
}

int set_raw(int fd) {
    termios settings{};
    if (tcgetattr(fd, &settings) != 0) {
        return -1;
    }
    cfmakeraw(&settings);
    cfsetispeed(&settings, B115200);
    cfsetospeed(&settings, B115200);
    return tcsetattr(fd, TCSANOW, &settings);
}

int open_device(const std::string& path) {
    const int fd = open(path.c_str(), O_RDWR | O_NOCTTY);
    if (fd < 0) {
        return -1;
    }
    if (set_raw(fd) != 0) {
        close(fd);
        return -1;
    }
    return fd;
}

int bind_server(int fd, const std::string& path) {
    sockaddr_un address{};
    address.sun_family = AF_UNIX;
    if (path.size() >= sizeof(address.sun_path)) {
        errno = EINVAL;
        return -1;
    }
    std::strcpy(address.sun_path, path.c_str());
    unlink(path.c_str());
    if (bind(fd, reinterpret_cast<sockaddr*>(&address), sizeof(address)) != 0) {
        return -1;
    }
    return chmod(path.c_str(), 0666);
}

int create_server(const std::string& path) {
    const int fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) {
        return -1;
    }
    if (bind_server(fd, path) != 0) {
        close(fd);
        return -1;
    }
    if (listen(fd, 1) != 0) {
        close(fd);
        return -1;
    }
    return fd;
}

bool wait_for_activity(pollfd* descriptors) {
    const int ready = poll(descriptors, 2, -1);
    if (ready < 0) {
        return errno == EINTR;
    }
    return ready > 0;
}

bool write_all(int fd, const char* data, size_t size) {
    while (size > 0) {
        const ssize_t written = write(fd, data, size);
        if (written < 0 && errno == EINTR) {
            continue;
        }
        if (written <= 0) {
            return false;
        }
        data += written;
        size -= static_cast<size_t>(written);
    }
    return true;
}

}  // namespace

Bridge::Bridge(Options options) : options_(std::move(options)) {}

int Bridge::run() {
    install_signals();
    const int server = create_server(options_.socket_path);
    if (server < 0) {
        return fail("socket");
    }
    const int device = open_device(options_.device_path);
    if (device < 0) {
        close(server);
        unlink(options_.socket_path.c_str());
        return fail("device");
    }
    std::cout << "listening on " << options_.socket_path << "\n" << std::flush;
    serve(server, device);
    close(device);
    close(server);
    unlink(options_.socket_path.c_str());
    return 0;
}

void Bridge::serve(int server, int device) {
    while (!g_stop) {
        const int client = accept(server, nullptr, nullptr);
        if (client < 0) {
            if (errno == EINTR) {
                continue;
            }
            std::perror("accept");
            return;
        }
        std::cout << "client connected\n" << std::flush;
        bridge_client(client, device);
        close(client);
        std::cout << "client disconnected\n" << std::flush;
    }
}

void Bridge::bridge_client(int client, int device) {
    pollfd descriptors[2] = {
        {client, POLLIN, 0},
        {device, POLLIN, 0},
    };
    while (!g_stop) {
        if (!wait_for_activity(descriptors)) {
            break;
        }
        if (!bridge_ready(descriptors[0], descriptors[1], client, device)) {
            break;
        }
    }
}

bool Bridge::bridge_ready(
    pollfd& client,
    pollfd& device,
    int client_fd,
    int device_fd
) {
    if (client.revents & (POLLHUP | POLLERR)) {
        return false;
    }
    if (device.revents & (POLLHUP | POLLERR)) {
        return false;
    }
    if (client.revents & POLLIN && !forward(client_fd, device_fd)) {
        return false;
    }
    if (device.revents & POLLIN && !forward(device_fd, client_fd)) {
        return false;
    }
    return true;
}

bool Bridge::forward(int from, int to) const {
    char buffer[4096];
    const ssize_t count = read(from, buffer, sizeof(buffer));
    if (count <= 0) {
        return false;
    }
    return write_all(to, buffer, static_cast<size_t>(count));
}

}  // namespace embed_emu_usb
