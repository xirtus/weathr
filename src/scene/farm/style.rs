use crate::scene::SceneContext;
use crossterm::style::Color;

#[derive(Clone, Copy)]
pub struct FarmSceneStyle {
    pub barn_roof: Color,
    pub barn_wall: Color,
    pub barn_door: Color,
    pub silo: Color,
    pub windmill: Color,
    pub crop_primary: Color,
    pub crop_secondary: Color,
    pub soil: Color,
    pub fence: Color,
}

impl FarmSceneStyle {
    pub fn resolve(ctx: &SceneContext<'_>) -> Self {
        let palette = ctx.palette;
        if ctx.conditions.sun.is_day {
            Self {
                barn_roof: Color::DarkRed,
                barn_wall: Color::Rgb { r: 160, g: 82, b: 45 },
                barn_door: Color::Rgb { r: 80, g: 40, b: 20 },
                silo: Color::Rgb { r: 180, g: 180, b: 160 },
                windmill: Color::Rgb { r: 120, g: 100, b: 80 },
                crop_primary: palette.ground_day,
                crop_secondary: Color::Rgb { r: 180, g: 160, b: 30 },
                soil: Color::Rgb { r: 101, g: 67, b: 33 },
                fence: Color::Rgb { r: 160, g: 130, b: 90 },
            }
        } else {
            Self {
                barn_roof: Color::DarkMagenta,
                barn_wall: Color::Rgb { r: 80, g: 40, b: 20 },
                barn_door: Color::Rgb { r: 40, g: 20, b: 10 },
                silo: Color::Rgb { r: 80, g: 80, b: 70 },
                windmill: Color::Rgb { r: 60, g: 50, b: 40 },
                crop_primary: palette.ground_night,
                crop_secondary: Color::Rgb { r: 80, g: 70, b: 10 },
                soil: Color::Rgb { r: 50, g: 30, b: 15 },
                fence: Color::Rgb { r: 70, g: 55, b: 35 },
            }
        }
    }
}
