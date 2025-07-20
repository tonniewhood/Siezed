
BUILD_DIR   := build
SRC_DIR     := src
INCLUDE_DIR := include
PKG_CONFIG  := pkg-config

CC          := gcc

# Common object files
COMMON_OBJS := $(BUILD_DIR)/tux_image.o

# Wayland protocol files
WAYLAND_PROTOCOL_OBJS := $(XDG_SHELL_OBJ)

# == X11 Stuff == #
X11_SRC    := $(SRC_DIR)/x11_window.c
X11_OBJ    := $(BUILD_DIR)/x11_window.o
X11_TARGET := $(BUILD_DIR)/x11_viewer
X11_CFLAGS := -std=c99 -Wall -Wextra -O2 -I$(INCLUDE_DIR) $(shell sdl2-config --cflags) $(shell pkg-config x11 --cflags)
X11_LDLIBS := $(shell sdl2-config --libs) -lSDL2_image $(shell pkg-config x11 --libs) 

# == Wayland Stuff == #
WAYLAND_PROTOCOLS_DIR := /usr/share/wayland-protocols/stable/xdg-shell
XDG_SHELL_XML := $(WAYLAND_PROTOCOLS_DIR)/xdg-shell.xml
XDG_SHELL_HEADER := $(BUILD_DIR)/xdg-shell-client-protocol.h
XDG_SHELL_CODE := $(BUILD_DIR)/xdg-shell-client-protocol.c
XDG_SHELL_OBJ := $(BUILD_DIR)/xdg-shell-client-protocol.o

WAYLAND_SRC    := $(SRC_DIR)/wayland_window.c
WAYLAND_OBJ    := $(BUILD_DIR)/wayland_window.o
WAYLAND_TARGET := $(BUILD_DIR)/wayland_viewer
WAYLAND_CFLAGS := -std=c99 -Wall -Wextra -O2 -I$(INCLUDE_DIR) -I$(BUILD_DIR) \
				  $(shell sdl2-config --cflags) \
				  $(shell ${PKG_CONFIG} --cflags wayland-client)
WAYLAND_LDLIBS := $(shell ${PKG_CONFIG} --libs wayland-client) \
				  -lSDL2_image \
				  $(shell sdl2-config --libs)

all: $(X11_TARGET) $(WAYLAND_TARGET)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(X11_TARGET): $(X11_OBJ) $(COMMON_OBJS)
	$(CC) $(X11_CFLAGS) -o $@ $^ $(X11_LDLIBS)

$(WAYLAND_TARGET): $(WAYLAND_OBJ) $(COMMON_OBJS) $(BUILD_DIR)/xdg-shell-client-protocol.o
	$(CC) $(WAYLAND_CFLAGS) -o $@ $^ $(WAYLAND_LDLIBS)

$(BUILD_DIR)/x11_window.o: $(SRC_DIR)/x11_window.c | $(BUILD_DIR)
	$(CC) $(X11_CFLAGS) -c $< -o $@

$(BUILD_DIR)/wayland_window.o: $(SRC_DIR)/wayland_window.c $(XDG_SHELL_HEADER) | $(BUILD_DIR)
	$(CC) $(WAYLAND_CFLAGS) -c $< -o $@

$(BUILD_DIR)/tux_image.o: $(SRC_DIR)/tux_image.c | $(BUILD_DIR)
	$(CC) $(X11_CFLAGS) -c $< -o $@

# Generate XDG shell protocol files
$(XDG_SHELL_HEADER): $(XDG_SHELL_XML) | $(BUILD_DIR)
	wayland-scanner client-header $< $@

$(XDG_SHELL_CODE): $(XDG_SHELL_XML) | $(BUILD_DIR)
	wayland-scanner private-code $< $@

$(XDG_SHELL_OBJ): $(XDG_SHELL_CODE) $(XDG_SHELL_HEADER) | $(BUILD_DIR)
	$(CC) $(WAYLAND_CFLAGS) -c $< -o $@

clean:
	rm -rf $(BUILD_DIR)

rebuild: clean all

.PHONY: all clean rebuild
