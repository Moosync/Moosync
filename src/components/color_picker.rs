// Adapted from https://github.com/laxsjo/lax-utils/

use leptos::{ev::*, html::*, logging::log, prelude::*};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorSpaceInfo {
    pub labels: (&'static str, &'static str, &'static str),
    pub units: (
        Option<&'static str>,
        Option<&'static str>,
        Option<&'static str>,
    ),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    Rgb,
    Hsl,
    Hsv,
}

impl ColorSpace {
    #[tracing::instrument(level = "trace", skip(self, components))]
    pub fn clamp_color_components(&self, components: (f32, f32, f32)) -> (f32, f32, f32) {
        let clamp = match self {
            ColorSpace::Rgb => Rgb::clamp_components,
            ColorSpace::Hsl => Hsl::clamp_components,
            ColorSpace::Hsv => Hsv::clamp_components,
        };

        clamp(components)
    }

    #[tracing::instrument(level = "trace", skip(self, floats))]
    fn floats_to_components(&self, floats: (f32, f32, f32)) -> (f32, f32, f32) {
        let convert = match self {
            ColorSpace::Rgb => Rgb::floats_to_components,
            ColorSpace::Hsl => Hsl::floats_to_components,
            ColorSpace::Hsv => Hsv::floats_to_components,
        };

        convert(floats)
    }

    #[tracing::instrument(level = "trace", skip(self, rgb))]
    fn color_components_from_rgb(&self, rgb: Rgb) -> (f32, f32, f32) {
        match self {
            ColorSpace::Rgb => Rgb::from_rgb(rgb).as_components(),
            ColorSpace::Hsl => Hsl::from_rgb(rgb).as_components(),
            ColorSpace::Hsv => Hsv::from_rgb(rgb).as_components(),
        }
    }
    #[tracing::instrument(level = "trace", skip(self, components))]
    fn rgb_from_color_components(&self, components: (f32, f32, f32)) -> Rgb {
        match self {
            ColorSpace::Rgb => Rgb::from_components(components).as_rgb(),
            ColorSpace::Hsl => Hsl::from_components(components).as_rgb(),
            ColorSpace::Hsv => Hsv::from_components(components).as_rgb(),
        }
    }
}

impl Display for ColorSpace {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ColorSpace::Rgb => "Rgb",
            ColorSpace::Hsl => "Hsl",
            ColorSpace::Hsv => "Hsv",
        })
    }
}

impl FromStr for ColorSpace {
    type Err = ();
    #[tracing::instrument(level = "trace", skip(s))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Rgb" => Ok(ColorSpace::Rgb),
            "Hsl" => Ok(ColorSpace::Hsl),
            "Hsv" => Ok(ColorSpace::Hsv),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DynamicColor {
    components: (f32, f32, f32),
    color_space: ColorSpace,
}

impl DynamicColor {
    #[tracing::instrument(level = "trace", skip(components, color_space))]
    pub fn new(components: (f32, f32, f32), color_space: ColorSpace) -> Self {
        Self {
            components: color_space.clamp_color_components(components),
            color_space,
        }
    }

    #[tracing::instrument(level = "trace", skip(floats, color_space))]
    pub fn from_floats(floats: (f32, f32, f32), color_space: ColorSpace) -> Self {
        Self {
            components: color_space.floats_to_components(floats),
            color_space,
        }
    }

    #[tracing::instrument(level = "trace", skip(color))]
    pub fn from_color<C: Color>(color: C) -> Self {
        Self::new(color.as_components(), C::COLOR_SPACE)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn to_color<C: Color>(self) -> C {
        C::from_components(self.set_color_space(C::COLOR_SPACE).components)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    #[tracing::instrument(level = "trace", skip(self, color_space))]
    pub fn set_color_space(mut self, color_space: ColorSpace) -> Self {
        let rgb = self.color_space.rgb_from_color_components(self.components);

        self.components = color_space.color_components_from_rgb(rgb);

        self.color_space = color_space;

        self
    }
}

pub trait Color {
    const COMPONENT_MAXES: (f32, f32, f32);
    const COLOR_SPACE: ColorSpace;

    fn as_components(&self) -> (f32, f32, f32);
    fn from_components(components: (f32, f32, f32)) -> Self
    where
        Self: Sized;

    fn as_floats(&self) -> (f32, f32, f32);
    fn from_floats(floats: (f32, f32, f32)) -> Self
    where
        Self: Sized;

    fn as_rgb(&self) -> Rgb;
    fn from_rgb(rgb: Rgb) -> Self
    where
        Self: Sized;

    fn clamp_components(components: (f32, f32, f32)) -> (f32, f32, f32)
    where
        Self: Sized,
    {
        let maxes = Self::COMPONENT_MAXES;
        (
            components.0.clamp(0., maxes.0 + 1.),
            components.1.clamp(0., maxes.1 + 1.),
            components.2.clamp(0., maxes.2 + 1.),
        )
    }

    fn components_to_floats(components: (f32, f32, f32)) -> (f32, f32, f32)
    where
        Self: Sized,
    {
        let maxes = Self::COMPONENT_MAXES;

        (
            components.0 / maxes.0,
            components.1 / maxes.1,
            components.2 / maxes.2,
        )
    }

    fn floats_to_components(floats: (f32, f32, f32)) -> (f32, f32, f32)
    where
        Self: Sized,
    {
        let maxes = Self::COMPONENT_MAXES;

        (
            (floats.0 * maxes.0).clamp(0., maxes.0 + 1.),
            (floats.1 * maxes.1).clamp(0., maxes.1 + 1.),
            (floats.2 * maxes.2).clamp(0., maxes.2 + 1.),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    #[tracing::instrument(level = "trace", skip(self))]
    /// Format rgb color as an hex code.
    ///
    /// The three hexadecimal components are returned without the usual hashtag
    /// prefix.
    /// E.g. `"f0f0f0"` instead of `"#f0f0f0"`.
    ///
    /// # Examples
    /// ```
    /// use lax_utils::color_picker::*;
    ///
    /// assert_eq!(Rgb::from_components((127., 127., 127.)).as_hex_code(), "7f7f7f".to_owned());
    ///
    /// assert_eq!(Rgb::from_components((0., 255., 0.)).as_hex_code(), "00ff00".to_owned());
    /// ```
    pub fn as_hex_code(self) -> String {
        let r = self.r as u8;
        let g = self.g as u8;
        let b = self.b as u8;

        format!("{:02x}{:02x}{:02x}", r, g, b)
    }

    #[tracing::instrument(level = "trace", skip(code))]
    /// Create a rgb color from a hex code.
    ///
    /// The code may begin with a hashtag.
    ///
    /// # Examples
    /// ```
    /// use lax_utils::color_picker::*;
    ///
    /// assert_eq!(Rgb::from_hex_code("7f7f7f"), Some(Rgb::from_components((127., 127., 127.))));
    /// assert_eq!(Rgb::from_hex_code("#00ff00"), Some(Rgb::from_components((0., 255., 0.))));
    ///
    /// assert_eq!(Rgb::from_hex_code("0f00"), None);
    /// assert_eq!(Rgb::from_hex_code("00fg00"), None);
    /// ```
    pub fn from_hex_code(code: &str) -> Option<Self> {
        let code = code.trim_start_matches('#');

        if code.len() != 6 {
            return None;
        }

        // code.char_indices().

        let binding = code.chars().collect::<Vec<_>>();
        let mut components = binding.chunks(2).filter_map(|chunk| {
            if let [a, b] = chunk {
                Some(a.to_digit(16)? * 16 + b.to_digit(16)?)
            } else {
                None
            }
        });

        Some(Rgb {
            r: components.next()? as f32,
            g: components.next()? as f32,
            b: components.next()? as f32,
        })
    }
}

impl Color for Rgb {
    const COMPONENT_MAXES: (f32, f32, f32) = (255., 255., 255.);
    const COLOR_SPACE: ColorSpace = ColorSpace::Rgb;

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_components(&self) -> (f32, f32, f32) {
        (self.r, self.g, self.b)
    }
    #[tracing::instrument(level = "trace", skip(components))]
    fn from_components(components: (f32, f32, f32)) -> Self {
        let components = ColorSpace::Rgb.clamp_color_components(components);

        Self {
            r: components.0,
            g: components.1,
            b: components.2,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_floats(&self) -> (f32, f32, f32) {
        (self.r / 255., self.g / 255., self.b / 255.)
    }
    #[tracing::instrument(level = "trace", skip(floats))]
    fn from_floats(floats: (f32, f32, f32)) -> Self {
        Self {
            r: floats.0.clamp(0., 1.) * 255.,
            g: floats.1.clamp(0., 1.) * 255.,
            b: floats.2.clamp(0., 1.) * 255.,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_rgb(&self) -> Rgb {
        *self
    }
    #[tracing::instrument(level = "trace", skip(rgb))]
    fn from_rgb(rgb: Rgb) -> Self {
        rgb
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Color for Hsl {
    const COMPONENT_MAXES: (f32, f32, f32) = (360., 100., 100.);
    const COLOR_SPACE: ColorSpace = ColorSpace::Hsl;

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_components(&self) -> (f32, f32, f32) {
        (self.h, self.s, self.l)
    }
    #[tracing::instrument(level = "trace", skip(components))]
    fn from_components(components: (f32, f32, f32)) -> Self {
        Self {
            h: components.0.clamp(0., 360.),
            s: components.1.clamp(0., 100.),
            l: components.2.clamp(0., 100.),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_floats(&self) -> (f32, f32, f32) {
        (self.h / 360., self.s / 100., self.l / 100.)
    }
    #[tracing::instrument(level = "trace", skip(floats))]
    fn from_floats(floats: (f32, f32, f32)) -> Self {
        Self {
            h: floats.0.clamp(0., 1.) * 360.,
            s: floats.1.clamp(0., 1.) * 100.,
            l: floats.2.clamp(0., 1.) * 100.,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Source: https://stackoverflow.com/a/9493060/15507414
    fn as_rgb(&self) -> Rgb {
        const ONE_THIRD: f32 = 1. / 3.;

        let (h, s, l) = self.as_floats();

        if s == 0. {
            let value = l * 255.;
            return Rgb::from_components((value, value, value));
        }

        #[tracing::instrument(level = "trace", skip(p, q, t))]
        /// What are `p`, `q`, and `t`? I have no idea :D
        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            const ONE_SIXTH: f32 = 1. / 6.;
            const ONE_HALF: f32 = 0.5;
            const TWO_THIRDS: f32 = 2. / 3.;

            if t < 0. {
                t += 1.
            };
            if t > 1. {
                t -= 1.
            };
            match t {
                _ if t < ONE_SIXTH => p + (q - p) * 6. * t,
                _ if t < ONE_HALF => q,
                _ if t < TWO_THIRDS => p + (q - p) * (TWO_THIRDS - t) * 6.,
                _ => p,
            }
        }

        let q = if l < 0.5 { l * (1. + s) } else { l + s - l * s };
        let p = 2. * l - q;

        // r, g, b are all floats in the range 0 to 1
        let r = hue_to_rgb(p, q, h + ONE_THIRD);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - ONE_THIRD);

        Rgb::from_floats((r, g, b))
    }

    #[tracing::instrument(level = "trace", skip(rgb))]
    /// Source: https://stackoverflow.com/a/9493060/15507414
    fn from_rgb(rgb: Rgb) -> Self {
        let (r, g, b) = rgb.as_floats();

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));

        let l = (max + min) / 2.;

        if max == min {
            // achromatic
            return Self::from_floats((0., 0., l));
        }

        let delta = max - min;

        let s = if l > 0.5 {
            delta / (2. - min - max)
        } else {
            delta / (max + min)
        };

        // matching on a float feels iffy...
        let h = match max {
            _ if max == r => (g - b) / delta + (if g < b { 6. } else { 0. }),
            _ if max == g => (b - r) / delta + 2.,
            _ if max == b => (r - g) / delta + 4.,
            _ => {
                panic!("float comparison failed for finding maximum component")
            }
        } / 6.;

        Self::from_floats((h, s, l))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Color for Hsv {
    const COMPONENT_MAXES: (f32, f32, f32) = (360., 100., 100.);
    const COLOR_SPACE: ColorSpace = ColorSpace::Hsv;

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_components(&self) -> (f32, f32, f32) {
        (self.h, self.s, self.v)
    }
    #[tracing::instrument(level = "trace", skip(components))]
    fn from_components(components: (f32, f32, f32)) -> Self {
        let components = Self::clamp_components(components);

        Self {
            h: components.0,
            s: components.1,
            v: components.2,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn as_floats(&self) -> (f32, f32, f32) {
        Self::components_to_floats(self.as_components())
    }
    #[tracing::instrument(level = "trace", skip(floats))]
    fn from_floats(floats: (f32, f32, f32)) -> Self {
        let components = Self::floats_to_components(floats);

        Self::from_components(components)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    /// Source: https://www.codespeedy.com/hsv-to-rgb-in-cpp/
    fn as_rgb(&self) -> Rgb {
        let (h, s, v) = self.as_components();

        if s == 0. {
            let adjusted_value = v / 100. * 255.;
            return Rgb::from_components((adjusted_value, adjusted_value, adjusted_value));
        }

        let (_, s_float, v_float) = self.as_floats();

        // I have no idea what any of the intermediary variables mean...
        let c = s_float * v_float;
        let x = c * (1. - ((h / 60. % 2.) - 1.).abs());
        let m = v_float - c;

        // These aren't really the final r, g, and b float components. See
        // below.
        let (r, g, b) = match () {
            _ if (0.0..=60.0).contains(&h) => (c, x, 0.),
            _ if (60.0..=120.0).contains(&h) => (x, c, 0.),
            _ if (120.0..=180.0).contains(&h) => (0., c, x),
            _ if (180.0..=240.0).contains(&h) => (0., x, c),
            _ if (240.0..=300.0).contains(&h) => (x, 0., c),
            _ => (c, 0., x),
        };

        Rgb::from_floats((r + m, g + m, b + m))
    }

    #[tracing::instrument(level = "trace", skip(rgb))]
    /// Source: https://www.rapidtables.com/convert/color/rgb-to-hsv.html
    /// Fixes from: https://mattlockyer.github.io/iat455/documents/rgb-hsv.pdf
    fn from_rgb(rgb: Rgb) -> Self
    where
        Self: Sized,
    {
        let (r, g, b) = rgb.as_floats();

        let max = r.max(g.max(b));
        let min = r.min(g.min(b));

        let delta = max - min;

        let h_degrees = if delta == 0. {
            0.
        } else if max == r {
            (g - b) / delta
        } else if max == g {
            (b - r) / delta + 2.
        } else {
            // blue is max
            (r - g) / delta + 4.
        }
        .rem_euclid(6.)
            * 60.;
        let h = h_degrees / 360.;

        let s = if max == 0. { 0. } else { delta / max };

        let v = max;

        Self::from_floats((h, s, v))
    }
}

#[tracing::instrument(level = "trace", skip(hex_code_setter, force_color, node_ref))]
#[component]
pub fn ColorPicker(
    #[prop()] hex_code_setter: impl Set<Value = String> + 'static,
    #[prop(default="#ffffff".into())] force_color: String,
    #[prop(optional)] node_ref: Option<NodeRef<Div>>,
) -> impl IntoView {
    const DECIMAL_PRECISION: usize = 2;

    let node_ref = node_ref.unwrap_or(create_node_ref());

    let normalised_inputs = create_rw_signal(false);

    let forced_color = Rgb::from_hex_code(force_color.as_str())
        .unwrap()
        .as_floats();

    let (color, set_color) = create_signal(DynamicColor::from_floats(
        (forced_color.0, forced_color.1, forced_color.2),
        ColorSpace::Rgb,
    ));

    let (color_hsv, set_color_hsv) = create_signal(color.get_untracked().to_color::<Hsv>());

    let (hex_code, set_hex_code) =
        create_signal(color.get_untracked().to_color::<Rgb>().as_hex_code());

    create_effect(move |_| {
        hex_code_setter.set(format!("#{}", hex_code.get()));
    });

    // create_tri

    let hue_float = Signal::derive(move || color_hsv.get().as_floats().0);
    let sat_float = Signal::derive(move || color_hsv.get().as_floats().1);
    let value_float = Signal::derive(move || color_hsv.get().as_floats().2);

    let update_with_hsv_floats = move |floats: (f32, f32, f32)| {
        let new_hsv = Hsv::from_floats((floats.0, floats.1, floats.2));
        if color_hsv.get_untracked() != new_hsv {
            set_color_hsv.set(new_hsv);
            let hsv = DynamicColor::from_color(color_hsv.get_untracked());
            set_color.set(hsv.set_color_space(color.get_untracked().color_space()));
            set_hex_code.set(hsv.to_color::<Rgb>().as_hex_code());
        }
    };

    let on_hue_float_change = move |hue: f32| {
        // set_color_hsv(color_hsv);
        update_with_hsv_floats((hue, sat_float.get_untracked(), value_float.get_untracked()));
    };
    let on_sat_float_change = move |sat: f32| {
        update_with_hsv_floats((hue_float.get_untracked(), sat, value_float.get_untracked()));
    };
    let on_value_float_change = move |value: f32| {
        update_with_hsv_floats((hue_float.get_untracked(), sat_float.get_untracked(), value));
    };

    view! {
        <div class="color-picker" class:normalised=normalised_inputs node_ref=node_ref>
            <div class="map">
                <SatValueSurface
                    sat=sat_float
                    set_sat=on_sat_float_change
                    value=value_float
                    set_value=on_value_float_change
                    hue=hue_float
                />
                <HueSlider hue=hue_float set_hue=on_hue_float_change />
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip(sat, set_sat, value, set_value, hue))]
#[component]
pub fn SatValueSurface<S, V>(
    #[prop(into)] sat: Signal<f32>,
    set_sat: S,
    #[prop(into)] value: Signal<f32>,
    set_value: V,
    #[prop(into)] hue: Signal<f32>,
) -> impl IntoView
where
    S: Fn(f32) + Copy + 'static,
    V: Fn(f32) + Copy + 'static,
{
    let (dragging, set_dragging) = create_signal(false);

    let custom_properties = move || {
        format!(
            "--cursor-x: {}; --cursor-y: {}; --current-hue: {};",
            sat.get(),
            1. - value.get(),
            hue.get(),
        )
    };

    let surface_ref = create_node_ref::<Div>();

    let on_pointer_move_color = move |ev: &PointerEvent| {
        let Some(surface_element) = surface_ref.get_untracked() else {
            log! {"Couldn't find element '.color-picker__color'!"};
            return;
        };

        let bounds = surface_element.get_bounding_client_rect();
        let element_x = bounds.left() as f32;
        let element_y = bounds.top() as f32;

        let width = surface_element.offset_width() as f32;
        let height = surface_element.offset_height() as f32;
        let global_x = ev.client_x() as f32;
        let global_y = ev.client_y() as f32;
        let x = ((global_x - element_x) / width).clamp(0., 1.);
        let y = ((global_y - element_y) / height).clamp(0., 1.);

        set_sat(x);
        set_value(1. - y);
    };

    let on_pointer_move = move |ev: PointerEvent| {
        if dragging.get_untracked() {
            on_pointer_move_color(&ev);
        }
    };
    // let on_pointer_move_color_closure =
    // wrap_closure_as_event_listener(on_pointer_move_color);

    let on_pointer_down = move |ev: PointerEvent| {
        set_dragging.set(true);
        on_pointer_move(ev);
    };

    view! {
        <div
            class="sat-value-surface"
            style=custom_properties
            on:pointerdown=on_pointer_down
            on:pointermove=on_pointer_move
            on:pointerup=move |_| set_dragging.set(false)
            node_ref=surface_ref
        >
            <div class="sat-value-surface__cursor" />
        </div>
    }
}

#[tracing::instrument(level = "trace", skip(hue, set_hue))]
#[component]
pub fn HueSlider<F>(#[prop(into)] hue: Signal<f32>, set_hue: F) -> impl IntoView
where
    F: Fn(f32) + Copy + 'static,
{
    let (dragging, set_dragging) = create_signal(false);

    let custom_properties = move || format!("--hue: {}", hue.get());

    let surface_ref = create_node_ref::<Div>();

    let on_pointer_move = move |ev: PointerEvent| {
        if !dragging.get_untracked() {
            return;
        }

        let Some(surface_element) = surface_ref.get() else {
            log! {"Couldn't find element '.hue-slider'!"};
            return;
        };

        let bounds = surface_element.get_bounding_client_rect();
        let element_x = bounds.left() as f32;

        let width = surface_element.offset_width() as f32;
        let global_x = ev.client_x() as f32;
        let x = ((global_x - element_x) / width).clamp(0., 1.);

        set_hue(x);
    };

    let on_pointer_down = move |ev: PointerEvent| {
        set_dragging.set(true);
        on_pointer_move(ev);
    };

    let on_pointer_up = move |_: _| {
        set_dragging.set(false);
    };

    view! {
        <div
            class="hue-slider"
            on:pointerdown=on_pointer_down
            on:pointermove=on_pointer_move
            on:pointerup=on_pointer_up
            node_ref=surface_ref
            style=custom_properties
        >
            <div class="hue-slider__cursor" />
        </div>
    }
}
