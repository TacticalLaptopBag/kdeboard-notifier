use std::collections::HashMap;
use std::sync::Arc;

use iced::font::{self, Font};
use iced::theme::{self, palette::Palette};
use iced::{Color, Theme};

pub struct KdeTheme {
    pub theme: Theme,
    pub font: Font,
}

impl Default for KdeTheme {
    fn default() -> Self {
        Self { theme: Theme::Light, font: Font::DEFAULT }
    }
}

pub fn load() -> KdeTheme {
    let Some(kg) = parse_kdeglobals() else {
        return KdeTheme::default();
    };

    let get = |section: &str, key: &str| -> Option<String> {
        kg.get(section)?.get(key).cloned()
    };

    let background = get("Colors:Window", "BackgroundNormal")
        .and_then(parse_color)
        .unwrap_or(Color::WHITE);
    let text = get("Colors:Window", "ForegroundNormal")
        .and_then(parse_color)
        .unwrap_or(Color::BLACK);
    let primary = get("Colors:Selection", "BackgroundNormal")
        .and_then(parse_color)
        .unwrap_or(Color::from_rgb8(61, 174, 233));
    let success = get("Colors:View", "ForegroundPositive")
        .and_then(parse_color)
        .unwrap_or(Color::from_rgb8(39, 174, 96));
    let danger = get("Colors:View", "ForegroundNegative")
        .and_then(parse_color)
        .unwrap_or(Color::from_rgb8(218, 68, 83));

    let palette = Palette { background, text, primary, success, danger };
    let theme = Theme::Custom(Arc::new(theme::Custom::new("KDE".into(), palette)));

    let font = get("General", "font")
        .as_deref()
        .and_then(parse_qt_font)
        .unwrap_or(Font::DEFAULT);

    KdeTheme { theme, font }
}

fn parse_kdeglobals() -> Option<HashMap<String, HashMap<String, String>>> {
    let path = dirs::config_dir()?.join("kdeglobals");
    let content = std::fs::read_to_string(path).ok()?;

    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            current = line[1..line.len() - 1].to_owned();
        } else if let Some((k, v)) = line.split_once('=') {
            sections
                .entry(current.clone())
                .or_default()
                .insert(k.trim().to_owned(), v.trim().to_owned());
        }
    }

    Some(sections)
}

fn parse_color(s: String) -> Option<Color> {
    let mut parts = s.split(',');
    let r: u8 = parts.next()?.trim().parse().ok()?;
    let g: u8 = parts.next()?.trim().parse().ok()?;
    let b: u8 = parts.next()?.trim().parse().ok()?;
    Some(Color::from_rgb8(r, g, b))
}

fn parse_qt_font(s: &str) -> Option<Font> {
    // Qt font descriptor: "Family,pointSize,-1,styleHint,weight,style,..."
    // Fields 4 and 5 carry weight and italic flag.
    let mut parts = s.splitn(10, ',');
    let family = parts.next()?.trim();
    let _pt_size = parts.next()?; // could wire into default_text_size if needed
    let _pixel_size = parts.next()?;
    let _style_hint = parts.next()?;
    let weight: u32 = parts.next()?.trim().parse().ok()?;
    let italic: u8 = parts.next()?.trim().parse().ok()?;

    // Qt weight: 0–99 mapped to thin–black; iced uses its own Weight enum.
    let iced_weight = match weight {
        0..=24 => font::Weight::Thin,
        25..=34 => font::Weight::ExtraLight,
        35..=44 => font::Weight::Light,
        45..=54 => font::Weight::Normal,
        55..=64 => font::Weight::Medium,
        65..=74 => font::Weight::Semibold,
        75..=84 => font::Weight::Bold,
        85..=94 => font::Weight::ExtraBold,
        _ => font::Weight::Black,
    };

    let style = if italic != 0 { font::Style::Italic } else { font::Style::Normal };

    // Leak to satisfy font::Family::Name(&'static str); safe for a config value
    // that lives for the entire process lifetime.
    let family: &'static str = Box::leak(family.to_owned().into_boxed_str());

    Some(Font {
        family: font::Family::Name(family),
        weight: iced_weight,
        stretch: font::Stretch::Normal,
        style,
    })
}
