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
}

impl BeachSceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        if ctx.conditions.sun.is_day {
            Self {
                sand: Color::Rgb { r: 210, g: 190, b: 130 },
                sand_dark: Color::Rgb { r: 180, g: 155, b: 100 },
                ocean: Color::Cyan,
                foam: Color::White,
                palm_trunk: Color::Rgb { r: 100, g: 70, b: 30 },
                palm_fronds: Color::Green,
            }
        } else {
            Self {
                sand: Color::Rgb { r: 100, g: 90, b: 60 },
                sand_dark: Color::Rgb { r: 70, g: 60, b: 40 },
                ocean: Color::DarkCyan,
                foam: Color::Grey,
                palm_trunk: Color::Rgb { r: 50, g: 35, b: 15 },
                palm_fronds: Color::DarkGreen,
            }
        }
    }
}
