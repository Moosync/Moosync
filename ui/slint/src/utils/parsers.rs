use slint::Color;

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_color(val: &str) -> Option<Color> {
    let val = val.trim();
    if let Some(hex) = val.strip_prefix('#') {
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::from_rgb_u8(r, g, b));
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Color::from_argb_u8(a, r, g, b));
            }
            _ => return None,
        }
    }
    if val.starts_with("rgb") {
        let start = val.find('(')? + 1;
        let end = val.rfind(')')?;
        let parts: Vec<&str> = val[start..end].split(',').map(|s| s.trim()).collect();
        if parts.len() < 3 {
            return None;
        }
        let r = parts[0].parse::<f32>().ok()? as u8;
        let g = parts[1].parse::<f32>().ok()? as u8;
        let b = parts[2].parse::<f32>().ok()? as u8;
        if parts.len() == 4 {
            let a = parts[3].parse::<f32>().ok()?;
            return Some(Color::from_argb_f32(
                a,
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            ));
        }
        return Some(Color::from_rgb_u8(r, g, b));
    }
    None
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if let Some(val) = val.strip_suffix("px") {
        return val.parse::<f32>().ok();
    }
    val.parse::<f32>().ok()
}
