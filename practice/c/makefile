
BUILD_DIR := build

CC     := gcc
CFLAGS := -std=c99 -Wall -Wextra -O2 $(shell sdl2-config --cflags) $(shell pkg-config x11 --cflags)
LDLIBS := $(shell sdl2-config --libs) -lSDL2_image $(shell pkg-config x11 --libs) 

SRC    := images.c
OBJ    := $(patsubst %.c, $(BUILD_DIR)/%.o, $(SRC))
TARGET := $(BUILD_DIR)/viewer

all: $(TARGET)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(TARGET): $(OBJ)
	$(CC) $(CFLAGS) -o $@ $^ $(LDLIBS)

$(BUILD_DIR)/%.o: %.c | $(BUILD_DIR)
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(BUILD_DIR)

rebuild: clean all

.PHONY: all clean rebuild
