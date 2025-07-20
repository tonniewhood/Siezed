
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

#include <SDL2/SDL.h>
#include <SDL2/SDL_image.h>

#include <X11/Xlib.h>
#include <X11/Xcms.h>

typedef uint8_t bool;
#define true 1
#define false 0

int main()
{

    Display *dpy = XOpenDisplay(NULL);
    if (!dpy)
    {
        fprintf(stderr, "Unable to open X display\n");
        return 1;
    }

    int screen = DefaultScreen(dpy);
    unsigned long black = BlackPixel(dpy, screen);
    XColor windowBackground;
    Colormap cmap = DefaultColormap(dpy, screen);
    if (!XParseColor(dpy, cmap, "#313131", &windowBackground) || !XAllocColor(dpy, cmap, &windowBackground))
    {
        fprintf(stderr, "Failed to allocate color\n");
        return 1;
    }

    Window win = XCreateSimpleWindow(
        dpy,
        RootWindow(dpy, screen),
        100, 100,
        761, 900,
        2,
        black,
        windowBackground.pixel);

    XStoreName(dpy, win, "Seized Practice Window");

    XSelectInput(dpy, win, ExposureMask | KeyPressMask);

    XColor foreground;
    if (!XParseColor(dpy, cmap, "#4169E1", &foreground) || !XAllocColor(dpy, cmap, &foreground))
    {
        fprintf(stderr, "Failed to allocate color\n");
        return 1;
    }

    XColor background;
    if (!XParseColor(dpy, cmap, "#000000", &background) || !XAllocColor(dpy, cmap, &background))
    {
        fprintf(stderr, "Failed to allocate color\n");
        return 1;
    }
    XGCValues gcvals;
    gcvals.foreground = foreground.pixel;
    gcvals.background = background.pixel;
    GC strings = XCreateGC(dpy, win, GCForeground | GCBackground, &gcvals);

    /* Set a larger font */
    XFontStruct *font = XLoadQueryFont(dpy, "-misc-fixed-bold-r-normal--20-200-75-75-c-100-iso8859-1");
    if (font)
    {
        XSetFont(dpy, strings, font->fid);
    }
    else
    {
        fprintf(stderr, "Failed to load font\n");
    }

    XGCValues frameVals;
    frameVals.line_width = 5;
    GC frame = XCreateGC(dpy, win, GCLineWidth, &frameVals);

    const int H = 800;
    const int W = 661;
    uint32_t *pixels = malloc(sizeof(*pixels) * W * H);
    if (!pixels)
    {
        fprintf(stderr, "Could not allocate for pixel buffer");
        return 1;
    }

    bool useImg = false;
    SDL_Surface *surf;
    if (SDL_Init(0) < 0)
    {
        fprintf(stderr, "SDL_Init: %s\n", SDL_GetError());
    }
    else if (!(IMG_Init(IMG_INIT_PNG) & IMG_INIT_PNG))
    {
        fprintf(stderr, "IMG_Init: %s\n", IMG_GetError());
    }
    else
    {
        SDL_Surface *png = IMG_Load("../../assets/images/tux.png");
        if (!png)
        {
            fprintf(stderr, "IMG_Load: %s\n", IMG_GetError());
        }
        else
        {
            surf = SDL_ConvertSurfaceFormat(png, SDL_PIXELFORMAT_RGBA32, 0);
            SDL_FreeSurface(png);
            if (!surf)
            {
                fprintf(stderr, "ConvertSurface: %s\n", SDL_GetError());
            }
            else
            {
                useImg = true;
            }
        }
    }

    for (int y = 0; y < H; y++)
    {
        for (int x = 0; x < W; x++)
        {
            uint8_t r = (uint8_t)(x * 255 / (W - 1));
            uint8_t g = (uint8_t)(y * 255 / (H - 1));
            uint8_t b = 0x80;
            pixels[y * W + x] = (r << 16) | (g << 8) | b;

            if (useImg && x < surf->w && y < surf->h)
            {
                // Get pointer to the pixel data (4 bytes per pixel for RGBA32)
                uint8_t *pixel_ptr = (uint8_t *)surf->pixels + y * surf->pitch + x * 4;

                // Read RGBA components (SDL_PIXELFORMAT_RGBA32 is R,G,B,A)
                uint8_t red = pixel_ptr[0];
                uint8_t green = pixel_ptr[1];
                uint8_t blue = pixel_ptr[2];
                uint8_t alpha = pixel_ptr[3];

                // Skip transparent pixels
                if (alpha == 0)
                    continue;

                // Convert to X11 format (typically BGRA or ARGB depending on endianness)
                // For most X11 systems, use ARGB format: 0xAARRGGBB
                pixels[y * W + x] = (alpha << 24) | (red << 16) | (green << 8) | blue;
            }
        }
    }

    XImage *img = XCreateImage(
        dpy, DefaultVisual(dpy, screen), DefaultDepth(dpy, screen),
        ZPixmap, 0, (char *)pixels, W, H, 32, 0);

    XMapWindow(dpy, win);

    XEvent ev;
    while (1)
    {
        XNextEvent(dpy, &ev);

        switch (ev.type)
        {
        case Expose:
            XDrawRectangle(
                dpy, win, frame,
                50, 50, 666, 806);
            XPutImage(
                dpy, win, DefaultGC(dpy, screen), img,
                0, 0, 53, 53, W, H);
            XFillRectangle(
                dpy, win, DefaultGC(dpy, screen),
                307, 283, 168, 30);
            XDrawString(
                dpy, win, strings,
                322, 305, "Tux Says Hello", 14);
            break;

        case KeyPress:
            XFreeGC(dpy, strings);
            XCloseDisplay(dpy);
            return 0;
        }
    }

    return 0;
}
