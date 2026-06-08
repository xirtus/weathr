use crate::scene::SceneContext;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct BeachSceneStyle {
    pub sand: Color,
    pub sand_dark: Color,
    pub ocean: Color,
    pub foam: Color,
    pub palm_trunk: Color,
    pub palm_fronds: Color,
    // Sky + skyline colors
    pub sky_bg: Color,
    pub skyline_distant: Color,
    pub skyline_mid: Color,
    pub skyline_near: Color,
    pub skyline_window: Color,
    pub skyline_roof: Color,
    pub skyline_accent: Color,
}

impl BeachSceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        let _palette = ctx.palette;
        if ctx.conditions.sun.is_day {
            Self {
                sand: Color::Rgb { r: 210, g: 190, b: 130 },
                sand_dark: Color::Rgb { r: 180, g: 155, b: 100 },
                ocean: Color::Cyan,
                foam: Color::White,
                palm_trunk: Color::Rgb { r: 100, g: 70, b: 30 },
                palm_fronds: Color::Green,
                sky_bg: Color::Rgb { r: 80, g: 180, b: 235 },
                skyline_distant: Color::Rgb { r: 175, g: 210, b: 235 },
                skyline_mid: Color::Rgb { r: 150, g: 185, b: 215 },
                skyline_near: Color::Rgb { r: 210, g: 195, b: 175 },
                skyline_window: Color::Rgb { r: 80, g: 210, b: 255 },
                skyline_roof: Color::Rgb { r: 180, g: 140, b: 100 },
                skyline_accent: Color::Rgb { r: 255, g: 150, b: 80 },
            }
        } else {
            Self {
                sand: Color::Rgb { r: 100, g: 90, b: 60 },
                sand_dark: Color::Rgb { r: 70, g: 60, b: 40 },
                ocean: Color::DarkCyan,
                foam: Color::Grey,
                palm_trunk: Color::Rgb { r: 50, g: 35, b: 15 },
                palm_fronds: Color::DarkGreen,
                sky_bg: Color::Rgb { r: 8, g: 12, b: 40 },
                skyline_distant: Color::Rgb { r: 90, g: 100, b: 140 },
                skyline_mid: Color::Rgb { r: 110, g: 115, b: 155 },
                skyline_near: Color::Rgb { r: 140, g: 130, b: 160 },
                skyline_window: Color::Rgb { r: 255, g: 200, b: 70 },
                skyline_roof: Color::Rgb { r: 150, g: 130, b: 110 },
                skyline_accent: Color::Rgb { r: 255, g: 140, b: 210 },
            }
        }
    }
}
