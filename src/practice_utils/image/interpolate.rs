use std::{cell::RefCell, rc::Rc};

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

    let (dst_w, dst_h) = if img_borrowed.locked_aspect_ratio {
        let scale = (new_width as f32 / img_borrowed.width as f32)
            .min(new_height as f32 / img_borrowed.height as f32);
        (
            (img_borrowed.width as f32 * scale).round() as usize,
            (img_borrowed.height as f32 * scale).round() as usize,
        )
    } else {
        (new_width, new_height)
    };

    if frame.resize(dst_w, dst_h) {
        return Ok(());
    }

    if img_borrowed.width == 0 || img_borrowed.height == 0 {
        eprintln!(
            "Image is empty ({}x{})",
            img_borrowed.width, img_borrowed.height
        );
        return Ok(());
    }

    let x_ratio = img_borrowed.width as f32 / dst_w as f32;
    let y_ratio = img_borrowed.height as f32 / dst_h as f32;

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
    for dest_row_idx in 0..frame.height {
        let src_y = (dest_row_idx as f32 * y_ratio)
            .floor()
            .clamp(0.0, img.height as f32 - 1.0) as usize;

        for dest_col_idx in 0..frame.width {
            let src_x = (dest_col_idx as f32 * x_ratio)
                .floor()
                .clamp(0.0, img.width as f32 - 1.0) as usize;

            frame.buffer[dest_row_idx * frame.width + dest_col_idx] =
                img.get_pixel(src_x, src_y).argb;
        }
    }

    Ok(())
}

pub fn bilinear_interpolation(
    frame: &mut Frame,
    img: &Image,
    x_ratio: f32,
    y_ratio: f32,
) -> anyhow::Result<()> {
    for dest_row_idx in 0..frame.height {
        let ideal_y_start = ((dest_row_idx as f32 + 0.5) * y_ratio) - 0.5;
        let y0 = ideal_y_start.floor().clamp(0.0, img.height as f32 - 1.0) as u32;
        let y1 = (y0 + 1).min(img.height as u32 - 1);
        let weight_y = ideal_y_start - ideal_y_start.floor();

        for dest_col_idx in 0..frame.width {
            let ideal_x_start = ((dest_col_idx as f32 + 0.5) * x_ratio) - 0.5;
            let x0 = ideal_x_start.floor().clamp(0.0, img.width as f32 - 1.0) as u32;
            let x1 = (x0 + 1).min(img.width as u32 - 1);
            let weight_x = ideal_x_start - ideal_x_start.floor();

            let pixel_00 = img.get_pixel(x0 as usize, y0 as usize);
            let pixel_10 = img.get_pixel(x1 as usize, y0 as usize);
            let pixel_01 = img.get_pixel(x0 as usize, y1 as usize);
            let pixel_11 = img.get_pixel(x1 as usize, y1 as usize);

            let top_blend = pixel_00.lerp(&pixel_10, weight_x);
            let bttm_blend = pixel_01.lerp(&pixel_11, weight_x);

            frame.buffer[dest_row_idx * frame.width + dest_col_idx] =
                top_blend.lerp(&bttm_blend, weight_y).argb;
        }
    }

    Ok(())
}
