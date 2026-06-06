mod style;

use crate::render::TerminalRenderer;
use crate::scene::{Scene, SceneContext, SceneLayout};
use crossterm::style::Color;
use std::io;
use style::MountainSceneStyle;

const PEAKS_ASCII: &str = include_str!("assets/peaks.txt");
const CABIN_ASCII: &str = include_str!("assets/cabin.txt");
const MATTERHORN_ASCII: &str = include_str!("assets/matterhorn.txt");
const MOOSE_ASCII: &str = include_str!("assets/moose.txt");

pub struct MountainScene {
    width: u16,
    height: u16,
}

impl MountainScene {
    const GROUND_HEIGHT: u16 = 5;

    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    fn render_art_colored(
        renderer: &mut TerminalRenderer,
        art: &str,
        x: u16,
        ground_y: u16,
        color_fn: impl Fn(char) -> Color,
    ) -> io::Result<()> {
        let lines: Vec<&str> = art.lines().collect();
        let height = lines.len() as u16;
        let start_y = ground_y.saturating_sub(height);
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    renderer.render_char(
                        x + col as u16,
                        start_y + row as u16,
                        ch,
                        color_fn(ch),
                    )?;
                }
            }
        }
        Ok(())
    }

    fn render_art_centered(
        renderer: &mut TerminalRenderer,
        art: &str,
        center_x: u16,
        ground_y: u16,
        color: Color,
    ) -> io::Result<()> {
        let lines: Vec<&str> = art.lines().collect();
        let art_w = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let art_h = lines.len() as u16;
        let start_x = center_x.saturating_sub(art_w / 2);
        let start_y = ground_y.saturating_sub(art_h);
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    renderer.render_char(start_x + col as u16, start_y + row as u16, ch, color)?;
                }
            }
        }
        Ok(())
    }

    fn render_pine(
        renderer: &mut TerminalRenderer,
        x: u16,
        ground_y: u16,
        color: Color,
    ) -> io::Result<()> {
        let pine = ["  /\\  ", " /##\\ ", "/####\\", "  ||  ", "  ||  "];
        let h = pine.len() as u16;
        let start_y = ground_y.saturating_sub(h);
        for (row, line) in pine.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    renderer.render_char(x + col as u16, start_y + row as u16, ch, color)?;
                }
            }
        }
        Ok(())
    }

    fn render_ground(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &MountainSceneStyle,
    ) -> io::Result<()> {
        let w = width as usize;
        for row in 0..Self::GROUND_HEIGHT {
            for col in 0..w {
                let r = pseudo_rand(col, row as usize);
                let (ch, color) = if row == 0 {
                    if r < 10 { ('*', style.peak_snow) }
                    else if r < 20 { ('.', style.ground_rock) }
                    else { ('^', style.ground) }
                } else {
                    (if r < 15 { '.' } else { ' ' }, style.ground)
                };
                renderer.render_char(col as u16, ground_y + row, ch, color)?;
            }
        }
        Ok(())
    }
}

fn pseudo_rand(x: usize, y: usize) -> u32 {
    ((x as u32 ^ 0xDEAD).wrapping_mul(y as u32 ^ 0xBEEF)) % 100
}

impl Scene for MountainScene {
    fn id(&self) -> &'static str {
        "mountain"
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
        let style = MountainSceneStyle::resolve(ctx);
        let layout = self.layout();
        let ground_y = layout.ground_y;
        let w = self.width;
        let center = w / 2;

        Self::render_ground(renderer, w, ground_y, &style)?;

        let city = ctx.city_name.map(|s| s.to_lowercase()).unwrap_or_default();
        let is_zermatt = city.contains("zermatt");
        let is_banff = city.contains("banff");

        if is_zermatt {
            // Matterhorn: distinctive steep peak
            Self::render_art_centered(renderer, MATTERHORN_ASCII, center, ground_y, Color::White)?;

            // Pine trees flanking
            let tree_offset = 18u16;
            if center > tree_offset {
                Self::render_pine(renderer, center.saturating_sub(tree_offset), ground_y, style.pine)?;
            }
            if center + tree_offset + 6 < w {
                Self::render_pine(renderer, center + tree_offset, ground_y, style.pine)?;
            }
        } else {
            // Default peaks for non-Zermatt mountains
            let peaks_lines: Vec<&str> = PEAKS_ASCII.lines().collect();
            let peaks_width = peaks_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
            let peaks_height = peaks_lines.len() as u16;
            let peaks_x = center.saturating_sub(peaks_width / 2);
            let peaks_y = ground_y.saturating_sub(peaks_height);

            let snow_chars = ['/', '\\', '*', '^'];
            for (row, line) in peaks_lines.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let color = if snow_chars.contains(&ch) && row < 3 {
                        style.peak_snow
                    } else if ch == 'X' || ch == 'x' {
                        style.peak_rock
                    } else {
                        style.slope
                    };
                    renderer.render_char(peaks_x + col as u16, peaks_y + row as u16, ch, color)?;
                }
            }

            // Cabin at base center
            let cabin_lines: Vec<&str> = CABIN_ASCII.lines().collect();
            let cabin_width = cabin_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
            let cabin_x = center.saturating_sub(cabin_width / 2);

            Self::render_art_colored(renderer, CABIN_ASCII, cabin_x, ground_y, |ch| match ch {
                '/' | '\\' | '^' => style.cabin_roof,
                '.' => Color::Cyan,
                _ => style.cabin_wall,
            })?;

            // Pine trees flanking cabin
            let tree_left_x = cabin_x.saturating_sub(8);
            let tree_right_x = cabin_x + cabin_width + 2;

            Self::render_pine(renderer, tree_left_x, ground_y, style.pine)?;
            if tree_right_x + 6 < w {
                Self::render_pine(renderer, tree_right_x, ground_y, style.pine)?;
            }

            if w > 100 {
                if tree_left_x > 14 {
                    Self::render_pine(renderer, tree_left_x - 8, ground_y, style.pine)?;
                }
                if tree_right_x + 14 < w {
                    Self::render_pine(renderer, tree_right_x + 8, ground_y, style.pine)?;
                }
            }
        }

        // Banff: moose silhouette on the left side
        if is_banff && w > 40 {
            Self::render_art_centered(renderer, MOOSE_ASCII, center / 3, ground_y, Color::DarkYellow)?;
        }

        Ok(())
    }
}
