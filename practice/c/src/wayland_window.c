#define _POSIX_C_SOURCE 200809L

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <sys/mman.h>
#include <errno.h>

#include <wayland-client.h>
#include <wayland-client-protocol.h>
#include <xdg-shell-client-protocol.h>

static struct wl_display *display = NULL;
static struct wl_registry *registry = NULL;
static struct wl_compositor *compositor = NULL;
static struct xdg_wm_base *xdg_wm_base = NULL;
static struct wl_shm *shm = NULL;

static struct wl_surface *surface = NULL;
static struct xdg_surface *xdg_surface = NULL;
static struct xdg_toplevel *xdg_toplevel = NULL;

// Buffer management
static struct wl_buffer *buffer = NULL;
static void *shm_data = NULL;
static int shm_fd = -1;
static size_t shm_size = 0;

// Window state
static int window_width = 800;
static int window_height = 600;
static bool configured = false;

// Create anonymous file for shared memory
static int create_anonymous_file(size_t size)
{
    char template[] = "/tmp/wayland-shm-XXXXXX";
    int fd = mkstemp(template);
    if (fd < 0)
    {
        perror("mkstemp failed");
        return -1;
    }

    // Remove the file so it's anonymous
    unlink(template);

    // Set the size
    if (ftruncate(fd, size) < 0)
    {
        perror("ftruncate failed");
        close(fd);
        return -1;
    }

    return fd;
}

// Create a buffer with colored background
static struct wl_buffer *create_buffer(int width, int height)
{
    int stride = width * 4; // 4 bytes per pixel (ARGB)
    size_t size = stride * height;

    // Create shared memory file
    shm_fd = create_anonymous_file(size);
    if (shm_fd < 0)
    {
        return NULL;
    }

    // Map the memory
    shm_data = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, shm_fd, 0);
    if (shm_data == MAP_FAILED)
    {
        perror("mmap failed");
        close(shm_fd);
        return NULL;
    }
    shm_size = size;

    // Fill with a nice blue color (ARGB format: 0xAARRGGBB)
    uint32_t *pixels = (uint32_t *)shm_data;
    uint32_t color = 0xFF4A90E2; // Opaque blue

    for (int i = 0; i < width * height; i++)
    {
        pixels[i] = color;
    }

    // Create Wayland shared memory pool
    struct wl_shm_pool *pool = wl_shm_create_pool(shm, shm_fd, size);
    struct wl_buffer *buffer = wl_shm_pool_create_buffer(pool, 0, width, height, stride, WL_SHM_FORMAT_ARGB8888);

    wl_shm_pool_destroy(pool);

    return buffer;
}

// Clean up buffer resources
static void cleanup_buffer()
{
    if (buffer)
    {
        wl_buffer_destroy(buffer);
        buffer = NULL;
    }

    if (shm_data && shm_data != MAP_FAILED)
    {
        munmap(shm_data, shm_size);
        shm_data = NULL;
    }

    if (shm_fd >= 0)
    {
        close(shm_fd);
        shm_fd = -1;
    }
}

// Buffer release callback - compositor tells us when it's done with the buffer
static void buffer_release(void *data, struct wl_buffer *wl_buffer)
{
    printf("Buffer released by compositor\n");
    // In a real app, you might want to reuse this buffer or mark it as available
}

static const struct wl_buffer_listener buffer_listener = {
    .release = buffer_release,
};

static void handle_wm_base_ping(void *data,
                                struct xdg_wm_base *wm_base,
                                uint32_t serial)
{
    xdg_wm_base_pong(wm_base, serial);
}

static const struct xdg_wm_base_listener wm_base_listener = {
    .ping = handle_wm_base_ping,
};

// XDG Surface configure handler
static void xdg_surface_configure(void *data,
                                  struct xdg_surface *xdg_surface,
                                  uint32_t serial)
{
    // Acknowledge the configure event
    xdg_surface_ack_configure(xdg_surface, serial);
    configured = true;

    printf("XDG Surface configured with serial: %u\n", serial);
}

static const struct xdg_surface_listener xdg_surface_listener = {
    .configure = xdg_surface_configure,
};

// XDG Toplevel configure handler
static void xdg_toplevel_configure(void *data,
                                   struct xdg_toplevel *xdg_toplevel,
                                   int32_t width,
                                   int32_t height,
                                   struct wl_array *states)
{
    printf("Toplevel configure: %dx%d\n", width, height);

    // If compositor suggests a size, use it
    if (width > 0 && height > 0)
    {
        window_width = width;
        window_height = height;
    }
    // Otherwise keep our default size
}

static void xdg_toplevel_close(void *data,
                               struct xdg_toplevel *xdg_toplevel)
{
    printf("Window close requested\n");
    // You could set a flag here to exit the main loop gracefully
}

static const struct xdg_toplevel_listener xdg_toplevel_listener = {
    .configure = xdg_toplevel_configure,
    .close = xdg_toplevel_close,
};

static void registry_handle_global(void *data,
                                   struct wl_registry *registry,
                                   uint32_t name,
                                   const char *interface,
                                   uint32_t version)
{
    if (strcmp(interface, wl_compositor_interface.name) == 0)
    {
        compositor = wl_registry_bind(registry, name, &wl_compositor_interface, 1);
    }
    else if (strcmp(interface, xdg_wm_base_interface.name) == 0)
    {
        xdg_wm_base = wl_registry_bind(registry, name, &xdg_wm_base_interface, 1);
        xdg_wm_base_add_listener(xdg_wm_base, &wm_base_listener, NULL);
    }
    else if (strcmp(interface, wl_shm_interface.name) == 0)
    {
        shm = wl_registry_bind(registry, name, &wl_shm_interface, 1);
    }
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_handle_global,
    .global_remove = NULL};

int main()
{
    display = wl_display_connect(NULL);
    if (!display)
    {
        perror("Failed to connect to Wayland Display\n");
        exit(2);
    }

    registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);

    // Wait for the registry events to be processed
    wl_display_dispatch(display);
    wl_display_roundtrip(display);

    // Check if we got the required objects
    if (!compositor)
    {
        fprintf(stderr, "Failed to get compositor\n");
        exit(1);
    }

    if (!xdg_wm_base)
    {
        fprintf(stderr, "Failed to get XDG WM base\n");
        exit(1);
    }

    if (!shm)
    {
        fprintf(stderr, "Failed to get shared memory\n");
        exit(1);
    }

    surface = wl_compositor_create_surface(compositor);
    xdg_surface = xdg_wm_base_get_xdg_surface(xdg_wm_base, surface);
    xdg_toplevel = xdg_surface_get_toplevel(xdg_surface);

    // Add event listeners
    xdg_surface_add_listener(xdg_surface, &xdg_surface_listener, NULL);
    xdg_toplevel_add_listener(xdg_toplevel, &xdg_toplevel_listener, NULL);

    // Set window properties
    xdg_toplevel_set_title(xdg_toplevel, "Wayland Window");
    xdg_toplevel_set_app_id(xdg_toplevel, "wayland-viewer");

    // Set initial window size (this is a hint to the compositor)
    xdg_toplevel_set_min_size(xdg_toplevel, 400, 300);
    xdg_toplevel_set_max_size(xdg_toplevel, 1200, 900);

    // Initial commit to trigger configure events
    wl_surface_commit(surface);

    // Wait for the initial configure event
    printf("Waiting for window configuration...\n");
    while (!configured && wl_display_dispatch(display) != -1)
    {
        // Keep processing events until we get configured
    }

    printf("Window configured! Size: %dx%d\n", window_width, window_height);

    // Create buffer with colored background
    printf("Creating buffer...\n");
    buffer = create_buffer(window_width, window_height);
    if (!buffer)
    {
        fprintf(stderr, "Failed to create buffer\n");
        exit(1);
    }

    // Add buffer listener
    wl_buffer_add_listener(buffer, &buffer_listener, NULL);

    // Attach buffer to surface and commit
    wl_surface_attach(surface, buffer, 0, 0);
    wl_surface_damage(surface, 0, 0, window_width, window_height);
    wl_surface_commit(surface);

    printf("Buffer attached and surface committed!\n");

    // Main event loop
    printf("Starting main event loop. Press Ctrl+C to exit.\n");
    while (wl_display_dispatch(display) != -1)
    {
        // Handle events from the compositor
        // In a real application, you'd also handle:
        // - Keyboard/mouse input
        // - Drawing/rendering
        // - Application logic
    }

    // Cleanup
    cleanup_buffer();
    xdg_toplevel_destroy(xdg_toplevel);
    xdg_surface_destroy(xdg_surface);
    wl_surface_destroy(surface);
    if (shm)
        wl_shm_destroy(shm);
    xdg_wm_base_destroy(xdg_wm_base);
    wl_compositor_destroy(compositor);
    wl_registry_destroy(registry);
    wl_display_disconnect(display);

    return 0;
}
