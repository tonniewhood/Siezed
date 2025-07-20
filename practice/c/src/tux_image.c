#include "../include/tux_image.h"
#include <stdio.h>
#include <stdlib.h>
#include <SDL2/SDL_image.h>

TuxImage load_tux_image(const char *image_path)
{
    TuxImage result = {0};

    // Initialize SDL
    if (SDL_Init(0) < 0)
    {
        fprintf(stderr, "SDL_Init: %s\n", SDL_GetError());
        return result;
    }

    // Initialize SDL_image
    if (!(IMG_Init(IMG_INIT_PNG) & IMG_INIT_PNG))
    {
        fprintf(stderr, "IMG_Init: %s\n", IMG_GetError());
        SDL_Quit();
        return result;
    }

    // Load the PNG image
    SDL_Surface *png = IMG_Load(image_path);
    if (!png)
    {
        fprintf(stderr, "IMG_Load: %s\n", IMG_GetError());
        IMG_Quit();
        SDL_Quit();
        return result;
    }

    // Convert to RGBA32 format
    SDL_Surface *surf = SDL_ConvertSurfaceFormat(png, SDL_PIXELFORMAT_RGBA32, 0);
    SDL_FreeSurface(png);

    if (!surf)
    {
        fprintf(stderr, "ConvertSurface: %s\n", SDL_GetError());
        IMG_Quit();
        SDL_Quit();
        return result;
    }

    // Allocate pixel buffer
    result.width = surf->w;
    result.height = surf->h;
    result.pixels = malloc(sizeof(uint32_t) * result.width * result.height);

    if (!result.pixels)
    {
        fprintf(stderr, "Failed to allocate memory for tux image pixels\n");
        SDL_FreeSurface(surf);
        IMG_Quit();
        SDL_Quit();
        return result;
    }

    // Copy and convert pixels from SDL surface to our format
    printf("Loading tux image: %dx%d pixels\n", result.width, result.height);
    printf("SDL Surface format: %s\n", SDL_GetPixelFormatName(surf->format->format));
    printf("Bytes per pixel: %d, Pitch: %d\n", surf->format->BytesPerPixel, surf->pitch);

    for (int y = 0; y < result.height; y++)
    {
        for (int x = 0; x < result.width; x++)
        {
            // Get pointer to the pixel data (4 bytes per pixel for RGBA32)
            uint8_t *pixel_ptr = (uint8_t *)surf->pixels + y * surf->pitch + x * 4;

            // Read RGBA components (SDL_PIXELFORMAT_RGBA32 is R,G,B,A)
            uint8_t red = pixel_ptr[0];
            uint8_t green = pixel_ptr[1];
            uint8_t blue = pixel_ptr[2];
            uint8_t alpha = pixel_ptr[3];

            // Convert to X11 ARGB format: 0xAARRGGBB
            result.pixels[y * result.width + x] = (alpha << 24) | (red << 16) | (green << 8) | blue;
        }
    }

    SDL_FreeSurface(surf);
    IMG_Quit();
    SDL_Quit();

    result.success = 1;
    printf("Successfully loaded tux image\n");

    return result;
}

void free_tux_image(TuxImage *tux_img)
{
    if (tux_img && tux_img->pixels)
    {
        free(tux_img->pixels);
        tux_img->pixels = NULL;
        tux_img->width = 0;
        tux_img->height = 0;
        tux_img->success = 0;
    }
}

void blend_tux_image(uint32_t *background_pixels, int bg_width, int bg_height,
                     const TuxImage *tux_img, int offset_x, int offset_y)
{
    if (!background_pixels || !tux_img || !tux_img->pixels || !tux_img->success)
    {
        return;
    }

    for (int y = 0; y < tux_img->height; y++)
    {
        for (int x = 0; x < tux_img->width; x++)
        {
            int bg_x = x + offset_x;
            int bg_y = y + offset_y;

            // Check bounds
            if (bg_x >= 0 && bg_x < bg_width && bg_y >= 0 && bg_y < bg_height)
            {
                uint32_t tux_pixel = tux_img->pixels[y * tux_img->width + x];
                uint8_t alpha = (tux_pixel >> 24) & 0xFF;

                // Skip fully transparent pixels
                if (alpha > 0)
                {
                    background_pixels[bg_y * bg_width + bg_x] = tux_pixel;
                }
            }
        }
    }
}
