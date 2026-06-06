use crate::theme::{Palette, Theme, ThemeRegistry};
use crossterm::style::Color;

pub const DEFAULT_PALETTE: Palette = Palette {
    sky_day: Color::Cyan,
    sky_night: Color::DarkBlue,
    ground_day: Color::Green,
    ground_night: Color::DarkGreen,
    accent_primary: Color::DarkRed,
    accent_secondary: Color::Rgb { r: 210, g: 180, b: 140 },
    atmosphere: None,
};

const CITY_PALETTE: Palette = Palette {
    sky_day: Color::Rgb { r: 135, g: 180, b: 220 },
    sky_night: Color::Rgb { r: 10, g: 10, b: 30 },
    ground_day: Color::Rgb { r: 60, g: 60, b: 60 },
    ground_night: Color::Rgb { r: 30, g: 30, b: 30 },
    accent_primary: Color::Rgb { r: 180, g: 100, b: 50 },
    accent_secondary: Color::Grey,
    atmosphere: None,
};

const FARM_PALETTE: Palette = Palette {
    sky_day: Color::Rgb { r: 180, g: 220, b: 255 },
    sky_night: Color::DarkBlue,
    ground_day: Color::Rgb { r: 100, g: 140, b: 40 },
    ground_night: Color::Rgb { r: 40, g: 60, b: 10 },
    accent_primary: Color::DarkRed,
    accent_secondary: Color::Rgb { r: 180, g: 150, b: 80 },
    atmosphere: None,
};

const BEACH_PALETTE: Palette = Palette {
    sky_day: Color::Rgb { r: 100, g: 190, b: 240 },
    sky_night: Color::Rgb { r: 10, g: 20, b: 60 },
    ground_day: Color::Rgb { r: 210, g: 190, b: 130 },
    ground_night: Color::Rgb { r: 100, g: 90, b: 60 },
    accent_primary: Color::Rgb { r: 255, g: 120, b: 30 },
    accent_secondary: Color::Cyan,
    atmosphere: Some(Color::Rgb { r: 0, g: 140, b: 200 }),
};

const MOUNTAIN_PALETTE: Palette = Palette {
    sky_day: Color::Rgb { r: 120, g: 170, b: 220 },
    sky_night: Color::Rgb { r: 5, g: 10, b: 40 },
    ground_day: Color::Rgb { r: 60, g: 100, b: 40 },
    ground_night: Color::Rgb { r: 20, g: 40, b: 15 },
    accent_primary: Color::Rgb { r: 150, g: 100, b: 60 },
    accent_secondary: Color::White,
    atmosphere: None,
};

const SANTA_CRUZ_PALETTE: Palette = Palette {
    sky_day: Color::Rgb { r: 100, g: 200, b: 240 },
    sky_night: Color::Rgb { r: 10, g: 20, b: 60 },
    ground_day: Color::Rgb { r: 210, g: 190, b: 130 },
    ground_night: Color::Rgb { r: 100, g: 90, b: 60 },
    accent_primary: Color::Red,
    accent_secondary: Color::Rgb { r: 255, g: 200, b: 50 },
    atmosphere: Some(Color::Cyan),
};

fn default_theme() -> Theme {
    Theme { id: "default", display_name: "Default", scene_id: "world", overlay_id: None, palette: DEFAULT_PALETTE }
}

fn city_theme() -> Theme {
    Theme { id: "city", display_name: "City", scene_id: "city", overlay_id: None, palette: CITY_PALETTE }
}

fn farm_theme() -> Theme {
    Theme { id: "farm", display_name: "Farm", scene_id: "farm", overlay_id: None, palette: FARM_PALETTE }
}

fn beach_theme() -> Theme {
    Theme { id: "beach", display_name: "Beach", scene_id: "beach", overlay_id: None, palette: BEACH_PALETTE }
}

fn mountain_theme() -> Theme {
    Theme { id: "mountain", display_name: "Mountain", scene_id: "mountain", overlay_id: None, palette: MOUNTAIN_PALETTE }
}

fn santa_cruz_theme() -> Theme {
    Theme { id: "santa_cruz", display_name: "Santa Cruz", scene_id: "santa_cruz", overlay_id: None, palette: SANTA_CRUZ_PALETTE }
}

pub fn register_all(registry: &mut ThemeRegistry) {
    registry.register(default_theme());
    registry.register(city_theme());
    registry.register(farm_theme());
    registry.register(beach_theme());
    registry.register(mountain_theme());
    registry.register(santa_cruz_theme());
}
