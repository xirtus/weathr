use crate::scene::SceneContext;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct MountainSceneStyle {
    pub peak_snow: Color,
    pub peak_rock: Color,
    pub slope: Color,
    pub cabin_roof: Color,
    pub cabin_wall: Color,
    pub pine: Color,
    pub ground: Color,
    pub ground_rock: Color,
}

impl MountainSceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        let palette = ctx.palette;
        if ctx.conditions.sun.is_day {
            Self {
                peak_snow: Color::White,
                peak_rock: Color::Grey,
                slope: Color::Rgb { r: 100, g: 90, b: 80 },
                cabin_roof: Color::DarkRed,
                cabin_wall: Color::Rgb { r: 139, g: 100, b: 60 },
                pine: Color::DarkGreen,
                ground: palette.ground_day,
                ground_rock: Color::Grey,
            }
        } else {
            Self {
                peak_snow: Color::Grey,
                peak_rock: Color::DarkGrey,
                slope: Color::Rgb { r: 50, g: 45, b: 40 },
                cabin_roof: Color::DarkMagenta,
                cabin_wall: Color::Rgb { r: 70, g: 50, b: 30 },
                pine: Color::Rgb { r: 0, g: 50, b: 0 },
                ground: palette.ground_night,
                ground_rock: Color::DarkGrey,
            }
        }
    }
}
