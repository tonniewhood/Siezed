use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use super::{Frame, Image};

pub enum InterpolationType {
    NearestNeighbor,
    Bilinear,
}

pub fn interpolate(
    frame: &mut Frame,
    img: Rc<RefCell<Image>>,
    new_width: usize,
    new_height: usize,
    interpolation_type: InterpolationType,
) -> anyhow::Result<()> {
    let img_borrowed = img.borrow();

    // Early exit if source image is empty
    if img_borrowed.width == 0 || img_borrowed.height == 0 {
        eprintln!(
            "Image is empty ({}x{})",
            img_borrowed.width, img_borrowed.height
        );
        return Ok(());
    }

    // Get effective dimensions accounting for rotation
    let (effective_width, effective_height) = img_borrowed.get_effective_dimensions();

    let (dst_w, dst_h) = if img_borrowed.locked_aspect_ratio {
        let scale = (new_width as f32 / effective_width as f32)
            .min(new_height as f32 / effective_height as f32);
        (
            (effective_width as f32 * scale).round() as usize,
            (effective_height as f32 * scale).round() as usize,
        )
    } else {
        (new_width, new_height)
    };

    frame.resize(dst_w, dst_h); // Resize the frame buffer

    let x_ratio = effective_width as f32 / dst_w as f32;
    let y_ratio = effective_height as f32 / dst_h as f32;

    match interpolation_type {
        InterpolationType::NearestNeighbor => {
            nearest_neighbor_interpolation(frame, &img_borrowed, x_ratio, y_ratio)
        }
        InterpolationType::Bilinear => {
            bilinear_interpolation(frame, &img_borrowed, x_ratio, y_ratio)
        }
    }
}

pub fn nearest_neighbor_interpolation(
    frame: &mut Frame,
    img: &Image,
    x_ratio: f32,
    y_ratio: f32,
) -> anyhow::Result<()> {
    let (effective_width, effective_height) = img.get_effective_dimensions();

    for dest_row_idx in 0..frame.height {
        let src_y = (dest_row_idx as f32 * y_ratio)
            .floor()
            .clamp(0.0, effective_height as f32 - 1.0) as usize;

        for dest_col_idx in 0..frame.width {
            let src_x = (dest_col_idx as f32 * x_ratio)
                .floor()
                .clamp(0.0, effective_width as f32 - 1.0) as usize;

            let pixel = img.get_rotated_pixel(src_x, src_y);
            frame.buffer[dest_row_idx * frame.width + dest_col_idx] =
                pixel.to_display_color(img.is_grayscale, img.inverted);
        }
    }

    Ok(())
}

fn lerp_coords(s: f32, max: u32) -> (usize, usize, f32) {
    // floor and clamp so we stay in [0..max-1]
    let s0_f = s.floor();
    let s0 = s0_f.clamp(0.0, max as f32 - 1.0) as u32;
    let s1 = (s0 + 1).min(max - 1);
    let w = s - s0_f;
    (s0 as usize, s1 as usize, w)
}

pub fn bilinear_interpolation(
    frame: &mut Frame,
    img: &Image,
    x_ratio: f32,
    y_ratio: f32,
) -> anyhow::Result<()> {
    let theta = img.rotation as f32 * PI / 180.0;
    let (sin_t, cos_t) = theta.sin_cos();
    let cx = (img.width as f32 - 1.0) * 0.5;
    let cy = (img.height as f32 - 1.0) * 0.5;
    let dcx = (frame.width as f32 - 1.0) * 0.5;
    let dcy = (frame.height as f32 - 1.0) * 0.5;

    for dest_row in 0..frame.height {
        for dest_col in 0..frame.width {
            let dx = dest_col as f32;
            let dy = dest_row as f32;

            let sx_scaled = (dx - dcx) * x_ratio;
            let sy_scaled = (dy - dcy) * y_ratio;

            let tx = cos_t * sx_scaled + sin_t * sy_scaled;
            let ty = -sin_t * sx_scaled + cos_t * sy_scaled;

            let sx = tx + cx;
            let sy = ty + cy;

            // 2) find sample indices + weights:
            let (x0, x1, weight_x) = lerp_coords(sx, img.width as u32);
            let (y0, y1, weight_y) = lerp_coords(sy, img.height as u32);

            let pixel_00 = img.get_pixel(x0 as usize, y0 as usize);
            let pixel_10 = img.get_pixel(x1 as usize, y0 as usize);
            let pixel_01 = img.get_pixel(x0 as usize, y1 as usize);
            let pixel_11 = img.get_pixel(x1 as usize, y1 as usize);

            // Apply grayscale conversion if needed, otherwise use original pixels
            let (p00, p10, p01, p11) = if img.is_grayscale || img.inverted {
                (
                    apply_pixel_effects(pixel_00, img.is_grayscale, img.inverted),
                    apply_pixel_effects(pixel_10, img.is_grayscale, img.inverted),
                    apply_pixel_effects(pixel_01, img.is_grayscale, img.inverted),
                    apply_pixel_effects(pixel_11, img.is_grayscale, img.inverted),
                )
            } else {
                (pixel_00, pixel_10, pixel_01, pixel_11)
            };

            let top_blend = p00.lerp(&p10, weight_x);
            let bttm_blend = p01.lerp(&p11, weight_x);

            frame.buffer[dest_row * frame.width + dest_col] =
                top_blend.lerp(&bttm_blend, weight_y).argb;
        }
    }

    Ok(())
}

fn apply_pixel_effects(
    mut pixel: super::Pixel,
    is_grayscale: bool,
    is_inverted: bool,
) -> super::Pixel {
    if is_grayscale {
        pixel = pixel.to_grayscale();
    }
    if is_inverted {
        pixel = pixel.to_inverted();
    }
    pixel
}
