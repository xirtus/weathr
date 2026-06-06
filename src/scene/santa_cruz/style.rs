use crate::scene::SceneContext;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct SantaCruzStyle {
    pub track: Color,
    pub track_support: Color,
    pub coaster_car: Color,
    pub coaster_person: Color,
    pub boardwalk: Color,
    pub sand: Color,
    pub sand_wet: Color,
    pub sign: Color,
}

impl SantaCruzStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        if ctx.conditions.sun.is_day {
            Self {
                track: Color::Rgb { r: 139, g: 90, b: 43 },
                track_support: Color::Rgb { r: 110, g: 70, b: 30 },
                coaster_car: Color::Red,
                coaster_person: Color::White,
                boardwalk: Color::Rgb { r: 180, g: 140, b: 80 },
                sand: Color::Rgb { r: 210, g: 190, b: 130 },
                sand_wet: Color::Rgb { r: 180, g: 155, b: 100 },
                sign: Color::Yellow,
            }
        } else {
            Self {
                track: Color::Rgb { r: 70, g: 45, b: 20 },
                track_support: Color::Rgb { r: 55, g: 35, b: 15 },
                coaster_car: Color::DarkRed,
                coaster_person: Color::Grey,
                boardwalk: Color::Rgb { r: 90, g: 70, b: 40 },
                sand: Color::Rgb { r: 100, g: 90, b: 60 },
                sand_wet: Color::Rgb { r: 70, g: 60, b: 40 },
                sign: Color::DarkYellow,
            }
        }
    }
}
