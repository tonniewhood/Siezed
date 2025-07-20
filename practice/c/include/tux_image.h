#ifndef TUX_IMAGE_H
#define TUX_IMAGE_H

#include <stdint.h>
#include <SDL2/SDL.h>

typedef struct
{
    uint32_t *pixels;
    int width;
    int height;
    int success;
} TuxImage;

/**
 * Load the tux image and convert it to a pixel buffer
 * @param image_path Path to the tux.png file
 * @return TuxImage structure with loaded pixel data
 */
TuxImage load_tux_image(const char *image_path);

/**
 * Free the memory allocated for the tux image
 * @param tux_img Pointer to the TuxImage structure
 */
void free_tux_image(TuxImage *tux_img);

/**
 * Blend the tux image onto a background pixel buffer
 * @param background_pixels Background pixel buffer
 * @param bg_width Background width
 * @param bg_height Background height
 * @param tux_img Tux image to blend
 * @param offset_x X offset where to place the tux image
 * @param offset_y Y offset where to place the tux image
 */
void blend_tux_image(uint32_t *background_pixels, int bg_width, int bg_height,
                     const TuxImage *tux_img, int offset_x, int offset_y);

#endif // TUX_IMAGE_H
