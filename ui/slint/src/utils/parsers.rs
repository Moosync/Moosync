use slint::Color;

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_color(val: &str) -> Option<Color> {
    let c = csscolorparser::parse(val).ok()?;
    let [r, g, b, a] = c.to_rgba8();
    Some(Color::from_argb_u8(a, r, g, b))
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if let Some(val) = val.strip_suffix("px") {
        return val.parse::<f32>().ok();
    }
    val.parse::<f32>().ok()
}
