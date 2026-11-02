use slint::Color;

use crate::RgbaColor;

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_color(val: &str) -> Option<Color> {
    let c = csscolorparser::parse(val).ok()?;
    let [r, g, b, a] = c.to_rgba8();
    Some(Color::from_argb_u8(a, r, g, b))
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn default_rgba() -> RgbaColor {
    RgbaColor {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_rgba_color(val: &str) -> RgbaColor {
    let Some(c) = csscolorparser::parse(val).ok() else {
        return default_rgba();
    };
    let [r, g, b, a] = c.to_rgba8();
    RgbaColor {
        r: r as f32,
        g: g as f32,
        b: b as f32,
        a: a as f32 / 255.0,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if let Some(val) = val.strip_suffix("px") {
        return val.parse::<f32>().ok();
    }
    val.parse::<f32>().ok()
}
