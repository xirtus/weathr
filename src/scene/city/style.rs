use crate::scene::SceneContext;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct CitySceneStyle {
    pub wall: Color,
    pub roof: Color,
    pub window_day: Color,
    pub window_night: Color,
    pub ground: Color,
    pub ground_line: Color,
    pub landmark: Color,
    pub antenna: Color,
}

impl CitySceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        let palette = ctx.palette;
        if ctx.conditions.sun.is_day {
            Self {
                wall: Color::Grey,
                roof: Color::DarkGrey,
                window_day: Color::Cyan,
                window_night: Color::Cyan,
                ground: Color::Rgb { r: 60, g: 60, b: 60 },
                ground_line: Color::DarkGrey,
                landmark: palette.accent_primary,
                antenna: Color::White,
            }
        } else {
            Self {
                wall: Color::Rgb { r: 40, g: 40, b: 55 },
                roof: Color::Rgb { r: 30, g: 30, b: 40 },
                window_day: Color::Yellow,
                window_night: Color::Yellow,
                ground: Color::Rgb { r: 30, g: 30, b: 30 },
                ground_line: Color::Rgb { r: 50, g: 50, b: 50 },
                landmark: Color::Rgb { r: 200, g: 150, b: 50 },
                antenna: Color::Grey,
            }
        }
    }

    pub fn window_color(&self, is_day: bool) -> Color {
        if is_day { self.window_day } else { self.window_night }
    }
}
