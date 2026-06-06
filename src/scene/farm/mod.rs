mod style;

use crate::render::TerminalRenderer;
use crate::scene::{Scene, SceneContext, SceneLayout};
use crossterm::style::Color;
use std::io;
use style::FarmSceneStyle;

const BARN_ASCII: &str = include_str!("assets/barn.txt");
const SILO_ASCII: &str = include_str!("assets/silo.txt");
const WINDMILL_ASCII: &str = include_str!("assets/windmill.txt");
const SUNFLOWER_ASCII: &str = include_str!("assets/sunflower.txt");

pub struct FarmScene {
    width: u16,
    height: u16,
    windmill_frame: usize,
}

impl FarmScene {
    const GROUND_HEIGHT: u16 = 6;

    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height, windmill_frame: 0 }
    }

    fn render_art(
        renderer: &mut TerminalRenderer,
        art: &str,
        x: u16,
        ground_y: u16,
        color: Color,
    ) -> io::Result<()> {
        let lines: Vec<&str> = art.lines().collect();
        let height = lines.len() as u16;
        let start_y = ground_y.saturating_sub(height);
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    renderer.render_char(x + col as u16, start_y + row as u16, ch, color)?;
                }
            }
        }
        Ok(())
    }

    fn render_sunflower(
        renderer: &mut TerminalRenderer,
        center_x: u16,
        ground_y: u16,
    ) -> io::Result<()> {
        let lines: Vec<&str> = SUNFLOWER_ASCII.lines().collect();
        let art_w = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let art_h = lines.len() as u16;
        let start_x = center_x.saturating_sub(art_w / 2);
        let start_y = ground_y.saturating_sub(art_h);

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' { continue; }
                let color = match ch {
                    '*' | '/' | '\\' => Color::Yellow,
                    '(' | ')' => Color::DarkYellow,
                    '|' => Color::DarkGreen,
                    _ => Color::Green,
                };
                renderer.render_char(start_x + col as u16, start_y + row as u16, ch, color)?;
            }
        }
        Ok(())
    }

    fn render_ground(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &FarmSceneStyle,
    ) -> io::Result<()> {
        let width = width as usize;
        for y in 0..Self::GROUND_HEIGHT {
            for x in 0..width {
                let r = pseudo_rand(x, y as usize);
                let (ch, color) = if y == 0 {
                    if r < 30 {
                        ('|', style.crop_secondary)
                    } else if r < 50 {
                        ('"', style.crop_secondary)
                    } else {
                        ('^', style.crop_primary)
                    }
                } else {
                    let ch = if r < 10 { '~' } else if r < 15 { '.' } else { ' ' };
                    (ch, style.soil)
                };
                renderer.render_char(x as u16, ground_y + y, ch, color)?;
            }
        }
        Ok(())
    }
}

fn pseudo_rand(x: usize, y: usize) -> u32 {
    ((x as u32 ^ 0x5DEE_CE6D).wrapping_mul(y as u32 ^ 0xB)) % 100
}

impl Scene for FarmScene {
    fn id(&self) -> &'static str {
        "farm"
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn layout(&self) -> SceneLayout {
        let ground_y = self.height.saturating_sub(Self::GROUND_HEIGHT + 1);
        SceneLayout {
            ground_y,
            chimney_pos: None,
            width: self.width,
            height: self.height,
        }
    }

    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()> {
        let style = FarmSceneStyle::resolve(ctx);
        let layout = self.layout();
        let ground_y = layout.ground_y;
        let w = self.width;
        let center = w / 2;

        Self::render_ground(renderer, w, ground_y, &style)?;

        let city = ctx.city_name.map(|s| s.to_lowercase()).unwrap_or_default();
        let is_kansas = city.contains("kansas");

        if is_kansas {
            // Sunflower field: sunflowers instead of barn for Kansas
            let spacing = 10u16;
            let count = (w / spacing).max(1);
            for i in 0..count {
                let sx = spacing / 2 + i * spacing;
                if sx < w {
                    Self::render_sunflower(renderer, sx, ground_y)?;
                }
            }
        } else {
            // Barn centered
            let barn_x = center.saturating_sub(10);
            Self::render_art(renderer, BARN_ASCII, barn_x, ground_y, style.barn_wall)?;

            // Silo to the right
            let silo_x = center + 14;
            if silo_x + 10 < w {
                Self::render_art(renderer, SILO_ASCII, silo_x, ground_y, style.silo)?;
            }

            // Windmill to the left
            if center > 30 {
                let wm_x = center.saturating_sub(28);
                Self::render_art(renderer, WINDMILL_ASCII, wm_x, ground_y, style.windmill)?;
            }
        }

        Ok(())
    }
}
