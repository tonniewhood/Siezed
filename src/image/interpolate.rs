use std::f32::consts::PI;

use crate::{canvas::PixelBuffer, color};

use super::Image;

pub enum InterpolationType {
    NearestNeighbor,
    Bilinear,
}

/// How the source image is transformed while it's being resampled
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewOptions {
    pub rotation: i16,
    pub grayscale: bool,
    pub inverted: bool,
}

/// Size of `img` once `rotation` is applied (90° and 270° swap width and height)
pub fn rotated_size(img: &Image, rotation: i16) -> (usize, usize) {
    let (width, height) = img.get_size();
    match rotation.rem_euclid(180) {
        0 => (width as usize, height as usize),
        _ => (height as usize, width as usize),
    }
}

/// Resamples `src` so that it fills all of `dst`. The scale factor comes from
/// the size difference, so resize `dst` to the target size before calling.
pub fn interpolate(
    src: &Image,
    dst: &mut PixelBuffer,
    opts: &ViewOptions,
    interpolation_type: InterpolationType,
) -> anyhow::Result<()> {
    // Early exit if source image is empty
    let (img_width, img_height) = src.get_size();
    if img_width == 0 || img_height == 0 {
        eprintln!("Image is empty ({}x{})", img_width, img_height);
        return Ok(());
    }

    let (dst_w, dst_h) = dst.get_size();
    if dst_w == 0 || dst_h == 0 {
        return Ok(());
    }

    // Get effective dimensions accounting for rotation
    let (effective_width, effective_height) = rotated_size(src, opts.rotation);
    let x_ratio = effective_width as f32 / dst_w as f32;
    let y_ratio = effective_height as f32 / dst_h as f32;

    match interpolation_type {
        InterpolationType::NearestNeighbor => {
            nearest_neighbor_interpolation(src, dst, opts, x_ratio, y_ratio)
        }
        InterpolationType::Bilinear => bilinear_interpolation(src, dst, opts, x_ratio, y_ratio),
    }
}

/// Maps a coordinate in the rotated image back to the source image.
/// Only handles multiples of 90°; rotates the same direction as the bilinear path.
fn unrotate_coords(
    x: usize,
    y: usize,
    effective_width: usize,
    effective_height: usize,
    rotation: i16,
) -> (usize, usize) {
    match rotation.rem_euclid(360) {
        90 => (y, effective_width - 1 - x),
        180 => (effective_width - 1 - x, effective_height - 1 - y),
        270 => (effective_height - 1 - y, x),
        _ => (x, y), // 0°, plus a fallback for non-90 degree rotations
    }
}

pub fn nearest_neighbor_interpolation(
    src: &Image,
    dst: &mut PixelBuffer,
    opts: &ViewOptions,
    x_ratio: f32,
    y_ratio: f32,
) -> anyhow::Result<()> {
    let (effective_width, effective_height) = rotated_size(src, opts.rotation);
    let (dst_w, dst_h) = dst.get_size();
    let dst_pixels = dst.buff_mut();

    for dest_row in 0..dst_h {
        let src_y = (dest_row as f32 * y_ratio)
            .floor()
            .clamp(0.0, effective_height as f32 - 1.0) as usize;

        for dest_col in 0..dst_w {
            let src_x = (dest_col as f32 * x_ratio)
                .floor()
                .clamp(0.0, effective_width as f32 - 1.0) as usize;

            let (img_x, img_y) = unrotate_coords(
                src_x,
                src_y,
                effective_width,
                effective_height,
                opts.rotation,
            );
            let pixel = src.get_pixel(img_x, img_y);
            dst_pixels[dest_row * dst_w + dest_col] =
                color::to_display_color(opts.grayscale, opts.inverted, pixel);
        }
    }

    Ok(())
}

fn lerp_coords(s: f32, max: u32) -> (usize, usize, f32) {
    // Clamp before splitting into index + weight, otherwise a sample just outside
    // the image (e.g. -0.00001 from float error) gets weight ~1 toward the wrong pixel
    let s = s.clamp(0.0, max as f32 - 1.0);
    let s0_f = s.floor();
    let s0 = s0_f as u32;
    let s1 = (s0 + 1).min(max - 1);
    let w = s - s0_f;
    (s0 as usize, s1 as usize, w)
}

pub fn bilinear_interpolation(
    src: &Image,
    dst: &mut PixelBuffer,
    opts: &ViewOptions,
    x_ratio: f32,
    y_ratio: f32,
) -> anyhow::Result<()> {
    let theta = opts.rotation as f32 * PI / 180.0;
    let (sin_t, cos_t) = theta.sin_cos();
    let (img_width, img_height) = src.get_size();
    let (dst_w, dst_h) = dst.get_size();
    let cx = (img_width as f32 - 1.0) * 0.5;
    let cy = (img_height as f32 - 1.0) * 0.5;
    let dcx = (dst_w as f32 - 1.0) * 0.5;
    let dcy = (dst_h as f32 - 1.0) * 0.5;

    let display = |pixel| color::to_display_color(opts.grayscale, opts.inverted, pixel);
    let dst_pixels = dst.buff_mut();

    for dest_row in 0..dst_h {
        for dest_col in 0..dst_w {
            let dx = dest_col as f32;
            let dy = dest_row as f32;

            let sx_scaled = (dx - dcx) * x_ratio;
            let sy_scaled = (dy - dcy) * y_ratio;

            let tx = cos_t * sx_scaled + sin_t * sy_scaled;
            let ty = -sin_t * sx_scaled + cos_t * sy_scaled;

            let sx = tx + cx;
            let sy = ty + cy;

            // 2) find sample indices + weights:
            let (x0, x1, weight_x) = lerp_coords(sx, img_width);
            let (y0, y1, weight_y) = lerp_coords(sy, img_height);

            // Apply grayscale/inversion before blending so the blend sees display colors
            let p00 = display(src.get_pixel(x0, y0));
            let p10 = display(src.get_pixel(x1, y0));
            let p01 = display(src.get_pixel(x0, y1));
            let p11 = display(src.get_pixel(x1, y1));

            let top_blend = color::lerp(p00, p10, weight_x);
            let bttm_blend = color::lerp(p01, p11, weight_x);

            dst_pixels[dest_row * dst_w + dest_col] = color::lerp(top_blend, bttm_blend, weight_y);
        }
    }

    Ok(())
}
