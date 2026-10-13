pub fn to_grayscale(value: u32) -> u32 {
    // Use luminance formula: 0.299*R + 0.587*G + 0.114*B
    let a = ((0xFF000000 & value) >> 24) as f32;
    let r = ((0x00FF0000 & value) >> 16) as f32;
    let g = ((0x0000FF00 & value) >> 8) as f32;
    let b = ((0x000000FF & value) >> 0) as f32;
    let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
    (a as u32) << 24 | (gray as u32) << 16 | (gray as u32) << 8 | (gray as u32)
}

pub fn to_display_color(is_grayscale: bool, is_inverted: bool, pixel: u32) -> u32 {
    let mut ret_pixel = pixel;

    // Apply grayscale transformation first
    if is_grayscale {
        ret_pixel = to_grayscale(ret_pixel);
    }

    // Apply inversion transformation
    if is_inverted {
        ret_pixel = to_inverted(ret_pixel);
    }

    ret_pixel
}

pub fn to_inverted(value: u32) -> u32 {
    let a = ((0xFF000000 & value) >> 24) as u8;
    let r = ((0x00FF0000 & value) >> 16) as u8;
    let g = ((0x0000FF00 & value) >> 8) as u8;
    let b = ((0x000000FF & value) >> 0) as u8;
    let r_inv = 255 - r;
    let g_inv = 255 - g;
    let b_inv = 255 - b;
    (a as u32) << 24 | (r_inv as u32) << 16 | (g_inv as u32) << 8 | (b_inv as u32)
}

/// Linearly interpolates every channel (alpha included) from `from` to `to`.
/// A `weight` of 0.0 returns `from`, 1.0 returns `to`.
pub fn lerp(from: u32, to: u32, weight: f32) -> u32 {
    let lerp_channel = |shift: u32| {
        let f = ((from >> shift) & 0xFF) as f32;
        let t = ((to >> shift) & 0xFF) as f32;
        (f * (1.0 - weight) + t * weight).round().clamp(0.0, 255.0) as u32
    };

    (lerp_channel(24) << 24) | (lerp_channel(16) << 16) | (lerp_channel(8) << 8) | lerp_channel(0)
}

/// Blends `src` over `dst` using the alpha stored in `src`'s top byte
/// (0 = transparent, 255 = opaque). Returns a 0RGB color.
pub fn blend_over(src: u32, dst: u32) -> u32 {
    let alpha = src >> 24;
    let inv_alpha = 255 - alpha;
    let blend_channel = |shift: u32| {
        let s = (src >> shift) & 0xFF;
        let d = (dst >> shift) & 0xFF;
        // +127 rounds to nearest instead of truncating
        (s * alpha + d * inv_alpha + 127) / 255
    };

    (blend_channel(16) << 16) | (blend_channel(8) << 8) | blend_channel(0)
}

#[cfg(test)]
mod tests {
    use super::{blend_over, lerp};

    #[test]
    fn lerp_endpoints_and_midpoint() {
        assert_eq!(lerp(0x00_00_00_00, 0xFF_FF_FF_FF, 0.0), 0x00_00_00_00);
        assert_eq!(lerp(0x00_00_00_00, 0xFF_FF_FF_FF, 1.0), 0xFF_FF_FF_FF);
        assert_eq!(lerp(0x00_00_00_00, 0xFF_FF_FF_FF, 0.5), 0x80_80_80_80);
    }

    #[test]
    fn opaque_replaces_destination() {
        assert_eq!(blend_over(0xFF_12_34_56, 0x00_AB_CD_EF), 0x00_12_34_56);
    }

    #[test]
    fn transparent_keeps_destination() {
        assert_eq!(blend_over(0x00_12_34_56, 0x00_AB_CD_EF), 0x00_AB_CD_EF);
    }

    #[test]
    fn half_alpha_mixes_evenly() {
        assert_eq!(blend_over(0x80_FF_FF_FF, 0x00_00_00_00), 0x00_80_80_80);
    }
}
