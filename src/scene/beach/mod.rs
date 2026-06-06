mod style;

use crate::render::TerminalRenderer;
use crate::scene::{Scene, SceneContext, SceneLayout};
use crossterm::style::Color;
use std::io;
use style::BeachSceneStyle;

const PALM_ASCII: &str = include_str!("assets/palm.txt");
const TIKI_ASCII: &str = include_str!("assets/tiki.txt");

pub struct BeachScene {
    width: u16,
    height: u16,
}

impl BeachScene {
    pub const GROUND_HEIGHT: u16 = 6;

    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn render_ground(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &BeachSceneStyle,
    ) -> io::Result<()> {
        let w = width as usize;
        for row in 0..Self::GROUND_HEIGHT {
            for col in 0..w {
                let (ch, color) = if row == 0 {
                    if col % 8 < 4 { ('~', style.foam) } else { ('~', style.ocean) }
                } else if row == 1 {
                    (if col % 6 == 0 { '.' } else { '_' }, style.sand_dark)
                } else {
                    let r = pseudo_rand(col, row as usize);
                    let ch = if r < 5 { '.' } else if r < 10 { ',' } else { ' ' };
                    (ch, style.sand)
                };
                renderer.render_char(col as u16, ground_y + row, ch, color)?;
            }
        }
        Ok(())
    }

    fn render_palm(
        renderer: &mut TerminalRenderer,
        x: u16,
        ground_y: u16,
        style: &BeachSceneStyle,
    ) -> io::Result<()> {
        let lines: Vec<&str> = PALM_ASCII.lines().collect();
        let height = lines.len() as u16;
        let start_y = ground_y.saturating_sub(height);
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let color: Color = match ch {
                    '(' | ')' | '_' => style.palm_fronds,
                    _ => style.palm_trunk,
                };
                renderer.render_char(x + col as u16, start_y + row as u16, ch, color)?;
            }
        }
        Ok(())
    }

    fn render_tiki(
        renderer: &mut TerminalRenderer,
        center_x: u16,
        ground_y: u16,
    ) -> io::Result<()> {
        let lines: Vec<&str> = TIKI_ASCII.lines().collect();
        let art_w = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let art_h = lines.len() as u16;
        let start_x = center_x.saturating_sub(art_w / 2);
        let start_y = ground_y.saturating_sub(art_h);

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' { continue; }
                let color = match row {
                    0 => Color::DarkYellow,
                    1 | 2 | 3 => Color::Yellow,
                    4 => Color::DarkYellow,
                    _ => Color::DarkRed,
                };
                renderer.render_char(start_x + col as u16, start_y + row as u16, ch, color)?;
            }
        }
        Ok(())
    }
}

fn pseudo_rand(x: usize, y: usize) -> u32 {
    ((x as u32 ^ 0xABCD).wrapping_mul(y as u32 ^ 0x1234)) % 100
}

impl Scene for BeachScene {
    fn id(&self) -> &'static str {
        "beach"
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
        let style = BeachSceneStyle::resolve(ctx);
        let layout = self.layout();
        let ground_y = layout.ground_y;
        let w = self.width;

        Self::render_ground(renderer, w, ground_y, &style)?;

        let city = ctx.city_name.map(|s| s.to_lowercase()).unwrap_or_default();
        let is_hawaii = city.contains("honolulu") || city.contains("hawaii") || city.contains("hi");

        // Palm trees
        if w > 20 {
            Self::render_palm(renderer, 5, ground_y, &style)?;
        }
        if w > 40 {
            Self::render_palm(renderer, w.saturating_sub(18), ground_y, &style)?;
        }
        if w > 120 {
            Self::render_palm(renderer, w / 4, ground_y, &style)?;
            Self::render_palm(renderer, 3 * w / 4, ground_y, &style)?;
        }

        // Hawaii-specific: tiki totem center
        if is_hawaii {
            let center_x = w / 2;
            Self::render_tiki(renderer, center_x, ground_y)?;
            // Extra tiki on wide terminals
            if w > 100 {
                Self::render_tiki(renderer, center_x / 2, ground_y)?;
            }
        }

        Ok(())
    }
}
