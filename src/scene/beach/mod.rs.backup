mod style;

use crate::render::TerminalRenderer;
use crate::scene::{Scene, SceneContext, SceneLayout};
use crossterm::style::Color;
use std::io;
use style::BeachSceneStyle;

const PALM_ASCII: &str = include_str!("assets/palm.txt");
const TIKI_ASCII: &str = include_str!("assets/tiki.txt");
const SKYLINE_ASCII: &str = include_str!("assets/miami_skyline.txt");
const MALDIVES_ASCII: &str = include_str!("assets/maldives.txt");

pub struct BeachScene {
    width: u16,
    height: u16,
}

impl BeachScene {
    pub const GROUND_HEIGHT: u16 = 8;

    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Render the ASCII art skyline as a prominent backdrop across the horizon.
    /// Renders for ALL beach scenes. Miami gets extra foreground towers.
    fn render_skyline(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &BeachSceneStyle,
        is_miami: bool,
    ) -> io::Result<()> {
        if width < 30 {
            return Ok(());
        }
        let w = width;
        let horizon = ground_y.saturating_sub(1);

        // ── Primary: ASCII art skyline, centered and prominent ──
        let sky_art_lines: Vec<&str> = SKYLINE_ASCII.lines().collect();
        let art_h = sky_art_lines.len() as u16;
        let art_w = sky_art_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;

        // Render single copy of skyline art sitting on the horizon
        let art_start_y = horizon.saturating_sub(art_h);
        let offset_x = if w > art_w { (w - art_w) / 2 } else { 0 };

        for (row, line) in sky_art_lines.iter().enumerate() {
            let y = art_start_y + row as u16;
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' { continue; }
                let x = offset_x + col as u16;
                if x >= w { continue; }
                let color = if row < 5 {
                    style.skyline_distant
                } else if row < 10 {
                    style.skyline_mid
                } else if row < 16 {
                    style.skyline_near
                } else {
                    style.skyline_roof
                };
                renderer.render_char(x, y, ch, color)?;
            }
        }

        // ── Fill remaining sky above the art with stippled gradient ──
        let Color::Rgb { r: br, g: bg, b: bb } = style.sky_bg else {
            return Ok(());
        };
        for row in 0..art_start_y {
            let t = row as f32 / art_start_y.max(1) as f32;
            let r = (br as f32 * (0.3 + 0.7 * t)) as u8;
            let g = (bg as f32 * (0.3 + 0.7 * t)) as u8;
            let b = (bb as f32 * (0.3 + 0.7 * t)) as u8;
            let color = Color::Rgb { r, g, b };
            for col in 0..w {
                if (col as usize + row as usize * 3) % 7 < 2 {
                    renderer.render_char(col, row, '·', color)?;
                }
            }
        }

        // ── Miami extra: Art Deco tower silhouettes in front ──
        if is_miami {
            if w >= 40 {
                Self::draw_tower(renderer, w / 6, horizon, 8, style)?;
            }
            if w >= 60 {
                Self::draw_tower(renderer, w / 3, horizon, 10, style)?;
            }
            if w >= 80 {
                Self::draw_tower(renderer, w * 2 / 3, horizon, 9, style)?;
            }
            if w >= 100 {
                Self::draw_tower(renderer, w * 5 / 6, horizon, 7, style)?;
            }
        }

        Ok(())
    }

    /// Draw a simple stepped tower silhouette in front of the skyline art.
    fn draw_tower(
        renderer: &mut TerminalRenderer,
        cx: u16,
        base_y: u16,
        height: u16,
        style: &BeachSceneStyle,
    ) -> io::Result<()> {
        if cx < 2 || height < 4 {
            return Ok(());
        }
        let tower = style.skyline_near;
        let roof = style.skyline_accent;
        let win = style.skyline_window;

        let mut y = base_y;
        let stages: &[(u16, u16)] = &[(9, 2), (7, 2), (5, 1), (3, 1)];
        for &(w, h) in stages {
            if y < h {
                break;
            }
            let half = w / 2;
            for _ in 0..h {
                y -= 1;
                let lx = cx.saturating_sub(half);
                for c in lx..lx + w {
                    if c == lx || c == lx + w - 1 {
                        renderer.render_char(c, y, '│', tower)?;
                    } else {
                        // Occasional window
                        let ch = if (c - lx) % 3 == 1 { '▪' } else { '█' };
                        let clr = if (c - lx) % 3 == 1 { win } else { tower };
                        renderer.render_char(c, y, ch, clr)?;
                    }
                }
            }
            // Roof ledge
            let lx = cx.saturating_sub(half);
            for c in lx..lx + w {
                renderer.render_char(c, y, '▔', roof)?;
            }
        }
        // Spire top
        if y > 0 {
            y -= 1;
            renderer.render_char(cx, y, '│', roof)?;
        }
        if y > 0 {
            y -= 1;
            renderer.render_char(cx, y, '◆', Color::White)?;
        }

        Ok(())
    }

    pub fn render_ground(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &BeachSceneStyle,
    ) -> io::Result<()> {
        let w = width as usize;
        // Curved shoreline effect — the water pushes further in at center
        let center = w as i32 / 2;
        for row in 0..Self::GROUND_HEIGHT {
            for col in 0..w {
                // Curved beach: center of the shore bulges outward (lower on screen)
                let col_i = col as i32;
                let dist_from_center = (col_i - center).abs();
                let curve = (dist_from_center as f32 * 0.08) as u16;
                let effective_row = row.saturating_sub(curve.min(row));

                let (ch, color) = if effective_row == 0 {
                    if col % 8 < 4 { ('~', style.foam) } else { ('~', style.ocean) }
                } else if effective_row == 1 {
                    let r = pseudo_rand(col, row as usize);
                    let ch = if r < 30 { '~' } else if r < 50 { '.' } else { '_' };
                    (ch, style.sand_dark)
                } else if effective_row == 2 {
                    let r = pseudo_rand(col, row as usize);
                    let ch = if r < 8 { ',' } else if r < 15 { '`' } else { ' ' };
                    (ch, style.sand_dark)
                } else if effective_row == 3 || effective_row == 4 {
                    let r = pseudo_rand(col, row as usize);
                    let ch = if r < 3 { '·' } else if r < 8 { '.' } else { ' ' };
                    (ch, style.sand)
                } else if effective_row == 5 || effective_row == 6 {
                    let r = pseudo_rand(col, row as usize);
                    let ch = if r < 2 { '♧' } else if r < 6 { '.' } else if r < 10 { '\'' } else { ' ' };
                    (ch, style.sand)
                } else {
                    if col % 11 < 4 { ('\'', Color::Rgb { r: 50, g: 140, b: 40 }) }
                    else if col % 13 == 0 { ('♧', Color::Rgb { r: 70, g: 160, b: 50 }) }
                    else { (' ', style.sand) }
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
                if ch == ' ' { continue; }
                let color = match ch {
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

    fn render_beach_details(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
    ) -> io::Result<()> {
        let w = width as i32;
        let gy = ground_y as i32;

        if w > 30 {
            let tx = (w * 3 / 7).max(8) as u16;
            let ty = (gy + 3) as u16;
            renderer.render_char(tx, ty, '[', Color::Rgb { r: 255, g: 60, b: 60 })?;
            renderer.render_char(tx + 1, ty, '═', Color::White)?;
            renderer.render_char(tx + 2, ty, '═', Color::Rgb { r: 255, g: 100, b: 100 })?;
            renderer.render_char(tx + 3, ty, '═', Color::White)?;
            renderer.render_char(tx + 4, ty, ']', Color::Rgb { r: 255, g: 60, b: 60 })?;
        }
        if w > 60 {
            let sx = (w * 5 / 8) as u16;
            let sy = (gy + 2) as u16;
            renderer.render_char(sx, sy, '✶', Color::Rgb { r: 255, g: 150, b: 60 })?;
        }
        if w > 80 {
            let shx = (w / 4) as u16;
            let shy = (gy + 3) as u16;
            renderer.render_char(shx, shy, '@', Color::Rgb { r: 240, g: 220, b: 200 })?;
        }
        if w > 50 {
            let ux = (w * 4 / 5).max(20) as u16;
            let uy = (gy + 3) as u16;
            renderer.render_char(ux, uy, '│', Color::Rgb { r: 200, g: 160, b: 100 })?;
            renderer.render_char(ux.saturating_sub(2), uy.saturating_sub(1), '/', Color::Yellow)?;
            renderer.render_char(ux.saturating_sub(1), uy.saturating_sub(1), '▔', Color::Rgb { r: 255, g: 200, b: 50 })?;
            renderer.render_char(ux, uy.saturating_sub(1), '▔', Color::Rgb { r: 255, g: 120, b: 40 })?;
            renderer.render_char(ux + 1, uy.saturating_sub(1), '▔', Color::Rgb { r: 255, g: 80, b: 30 })?;
            renderer.render_char(ux + 2, uy.saturating_sub(1), '\\', Color::Yellow)?;
            renderer.render_char(ux, uy + 1, '▄', Color::Rgb { r: 200, g: 160, b: 100 })?;
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

        let city = ctx.city_name.map(|s| s.to_lowercase()).unwrap_or_default();
        let is_miami = city.contains("miami");
        let is_hawaii = city.contains("honolulu") || city.contains("hawaii") || city.contains("hi");
        let is_maldives = city.contains("maldives");

        // 1. Backdrop — Maldives gets its own scene, Hawaii gets open horizon
        if is_maldives {
            let lines: Vec<&str> = MALDIVES_ASCII.lines().collect();
            let art_h = lines.len() as u16;
            let art_w = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
            let start_y = ground_y.saturating_sub(art_h + 2);

            // Stippled sky above the art
            let Color::Rgb { r: br, g: bg, b: bb } = style.sky_bg else { return Ok(()); };
            for row in 0..start_y {
                let t = row as f32 / start_y.max(1) as f32;
                let color = Color::Rgb {
                    r: (br as f32 * (0.5 + 0.5 * t)) as u8,
                    g: (bg as f32 * (0.5 + 0.5 * t)) as u8,
                    b: (bb as f32 * (0.5 + 0.5 * t)) as u8,
                };
                for col in 0..w {
                    if (col as usize + row as usize * 3) % 7 < 2 {
                        renderer.render_char(col, row, '·', color)?;
                    }
                }
            }

            let offset_x = if w > art_w { (w - art_w) / 2 } else { 0 };
            for (row, line) in lines.iter().enumerate() {
                let y = start_y + row as u16;
                for (col, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let x = offset_x + col as u16;
                    if x >= w { continue; }
                    let color = match ch {
                        '~' | '^' | 'v' => style.ocean,
                        '"' | 'g' | 'y' | '-' | '_' => style.skyline_roof,
                        'H' | 'm' | 'H' => Color::DarkYellow,
                        '%' | 'J' | 'M' | 'L' | 'A' => style.skyline_near,
                        '|' | ',' | '.' => Color::DarkGrey,
                        'w' | 'W' => style.foam,
                        '`' | '\\' => Color::Rgb { r: 100, g: 140, b: 80 },
                        _ => style.skyline_distant,
                    };
                    renderer.render_char(x, y, ch, color)?;
                }
            }
        } else if !is_hawaii {
            Self::render_skyline(renderer, w, ground_y, &style, is_miami)?;
        }

        // 2. Beach ground (Maldives art includes its own shoreline)
        if !is_maldives {
            Self::render_ground(renderer, w, ground_y, &style)?;
        }

        // 3. Beach props (skip for Maldives — art is self-contained)
        if !is_maldives {
            Self::render_beach_details(renderer, w, ground_y)?;
        }

        // 4. Palm trees (skip for Maldives — art includes its own)
        if is_maldives {
            // Maldives art already has palms
        } else if is_miami && w >= 40 {
            Self::render_palm(renderer, 6, ground_y, &style)?;
            Self::render_palm(renderer, 19, ground_y, &style)?;
            Self::render_palm(renderer, w.saturating_sub(14), ground_y, &style)?;
            Self::render_palm(renderer, w.saturating_sub(27), ground_y, &style)?;
        } else {
            if w > 20 {
                Self::render_palm(renderer, 5, ground_y, &style)?;
            }
            if w > 40 {
                Self::render_palm(renderer, w.saturating_sub(18), ground_y, &style)?;
            }
        }
        if w > 120 {
            Self::render_palm(renderer, w / 4, ground_y, &style)?;
            Self::render_palm(renderer, 3 * w / 4, ground_y, &style)?;
        }

        // 5. Hawaii tiki totems
        if is_hawaii {
            let cx = w / 2;
            Self::render_tiki(renderer, cx, ground_y)?;
            if w > 100 {
                Self::render_tiki(renderer, cx / 2, ground_y)?;
            }
        }

        Ok(())
    }
}
