#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <wayland-client.h>
#include <wayland-client-protocol.h>
#include <xdg-shell-client-protocol.h>

static struct wl_display *display = NULL;
static struct wl_registry *registry = NULL;
static struct wl_compositor *compositor = NULL;
static struct xdg_wm_base *xdg_wm_base = NULL;

static struct wl_surface *surface = NULL;
static struct xdg_surface *xdg_surface = NULL;
static struct xdg_toplevel *xdg_toplevel = NULL;

static void handle_wm_base_ping(void *data,
                                struct xdg_wm_base *wm_base,
                                uint32_t serial)
{
    xdg_wm_base_pong(wm_base, serial);
}

static const struct xdg_wm_base_listener wm_base_listener = {
    .ping = handle_wm_base_ping,
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

    surface = wl_compositor_create_surface(compositor);
    xdg_surface = xdg_wm_base_get_xdg_surface(xdg_wm_base, surface);
    xdg_toplevel = xdg_surface_get_toplevel(xdg_surface);

    xdg_toplevel_set_title(xdg_toplevel, "Wayland Window");

    wl_surface_commit(surface);

    while (wl_display_dispatch(display) != 1)
        ;

    xdg_toplevel_destroy(xdg_toplevel);
    xdg_surface_destroy(xdg_surface);
    wl_surface_destroy(surface);
    xdg_wm_base_destroy(xdg_wm_base);
    wl_compositor_destroy(compositor);
    wl_registry_destroy(registry);
    wl_display_disconnect(display);

    return 0;
}
