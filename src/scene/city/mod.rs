mod style;

use crate::render::TerminalRenderer;
use crate::scene::{ChimneyPosition, Scene, SceneContext, SceneLayout};
use crossterm::style::Color;
use std::io;
use style::CitySceneStyle;

const NYC_ART: &str = include_str!("assets/nyc.txt");
const CHRYSLER_ART: &str = include_str!("assets/chrysler.txt");
const LONDON_ART: &str = include_str!("assets/london.txt");
const PARIS_ART: &str = include_str!("assets/paris.txt");
const GATE_ART: &str = include_str!("assets/gate.txt");
const OPERA_HOUSE_ART: &str = include_str!("assets/opera_house.txt");
const BURJ_ART: &str = include_str!("assets/burj.txt");
const TOKYO_TOWER_ART: &str = include_str!("assets/tokyo_tower.txt");
const TORII_ART: &str = include_str!("assets/torii.txt");
const PAGODA_ART: &str = include_str!("assets/pagoda.txt");
const CHRIST_ART: &str = include_str!("assets/christ.txt");
const STATUE_LIBERTY_ART: &str = include_str!("assets/statue_liberty.txt");
const TOWER_BRIDGE_ART: &str = include_str!("assets/tower_bridge.txt");
const ARC_ART: &str = include_str!("assets/arc.txt");
const TOKYO_SKYLINE_ART: &str = include_str!("assets/tokyo_skyline.txt");

pub struct CityScene {
    width: u16,
    height: u16,
    pub city_name: Option<String>,
}

impl CityScene {
    const GROUND_HEIGHT: u16 = 5;

    pub fn new(width: u16, height: u16, city_name: Option<String>) -> Self {
        Self { width, height, city_name }
    }

    fn render_building(
        renderer: &mut TerminalRenderer,
        x: u16,
        ground_y: u16,
        width: u16,
        building_height: u16,
        style: &CitySceneStyle,
        is_day: bool,
    ) -> io::Result<()> {
        if building_height == 0 || width < 2 {
            return Ok(());
        }
        let top_y = ground_y.saturating_sub(building_height);

        for col in x..x.saturating_add(width) {
            renderer.render_char(col, top_y, '_', style.roof)?;
        }

        for row in 0..building_height {
            let y = top_y + row;
            renderer.render_char(x.saturating_sub(1), y, '|', style.wall)?;
            renderer.render_char(x + width, y, '|', style.wall)?;

            if row % 2 == 0 && row > 0 {
                let win_color = style.window_color(is_day);
                let mut wx = x;
                while wx + 2 < x + width {
                    renderer.render_char(wx, y, '[', style.wall)?;
                    renderer.render_char(wx + 1, y, '_', win_color)?;
                    renderer.render_char(wx + 2, y, ']', style.wall)?;
                    wx += 4;
                }
            }
        }
        Ok(())
    }

    fn render_landmark(
        renderer: &mut TerminalRenderer,
        art: &str,
        center_x: u16,
        ground_y: u16,
        color: Color,
    ) -> io::Result<()> {
        let lines: Vec<&str> = art.lines().collect();
        let art_width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
        let art_height = lines.len() as u16;
        let start_x = center_x.saturating_sub(art_width / 2);
        let start_y = ground_y.saturating_sub(art_height);

        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    renderer.render_char(start_x + col as u16, start_y + row as u16, ch, color)?;
                }
            }
        }
        Ok(())
    }

    fn render_antenna(
        renderer: &mut TerminalRenderer,
        x: u16,
        top_y: u16,
        height: u16,
        color: Color,
    ) -> io::Result<()> {
        for i in 0..height {
            renderer.render_char(x, top_y.saturating_sub(i), '|', color)?;
        }
        renderer.render_char(x, top_y.saturating_sub(height), '*', color)?;
        Ok(())
    }

    fn render_ground(
        renderer: &mut TerminalRenderer,
        width: u16,
        ground_y: u16,
        style: &CitySceneStyle,
    ) -> io::Result<()> {
        for row in 0..Self::GROUND_HEIGHT {
            for col in 0..width {
                let ch = if row == 0 {
                    if col % 20 < 10 { '-' } else { ' ' }
                } else {
                    ' '
                };
                let color = if ch == '-' { style.ground_line } else { style.ground };
                renderer.render_char(col, ground_y + row, ch, color)?;
            }
        }
        Ok(())
    }
}

impl Scene for CityScene {
    fn id(&self) -> &'static str {
        "city"
    }

    fn update_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn layout(&self) -> SceneLayout {
        let ground_y = self.height.saturating_sub(Self::GROUND_HEIGHT + 1);
        let chimney_x = self.width / 2 + 2;
        SceneLayout {
            ground_y,
            chimney_pos: Some(ChimneyPosition { x: chimney_x, y: ground_y.saturating_sub(20) }),
            width: self.width,
            height: self.height,
        }
    }

    fn render(&self, renderer: &mut TerminalRenderer, ctx: &SceneContext<'_>) -> io::Result<()> {
        let style = CitySceneStyle::resolve(ctx);
        let is_day = ctx.conditions.sun.is_day;
        let layout = self.layout();
        let ground_y = layout.ground_y;
        let w = self.width;
        let cx = w / 2;
        let s = (w / 80).max(1); // scale factor

        Self::render_ground(renderer, w, ground_y, &style)?;

        let city = ctx
            .city_name
            .or(self.city_name.as_deref())
            .map(|n| n.to_lowercase())
            .unwrap_or_default();

        if city.contains("new york") || city.contains("nyc") {
            // ── New York ── dense Art Deco skyline
            // Empire State Building center
            Self::render_landmark(renderer, NYC_ART, cx, ground_y, style.landmark)?;
            // Chrysler Building right of Empire State
            Self::render_landmark(renderer, CHRYSLER_ART, cx + 16 * s, ground_y, Color::DarkYellow)?;

            // ── Right-side skyscraper canyon ──
            Self::render_building(renderer, cx + 26 * s, ground_y, 6 * s, 16, &style, is_day)?;
            Self::render_building(renderer, cx + 34 * s, ground_y, 5 * s, 11, &style, is_day)?;
            Self::render_building(renderer, cx + 41 * s, ground_y, 5 * s, 8, &style, is_day)?;
            Self::render_antenna(renderer, cx + 28 * s, ground_y.saturating_sub(16), 3, style.antenna)?;
            // Extra-wide: more right-side towers
            if w > 90 {
                Self::render_building(renderer, cx + 48 * s, ground_y, 4 * s, 13, &style, is_day)?;
                Self::render_building(renderer, cx + 54 * s, ground_y, 4 * s, 9, &style, is_day)?;
            }
            if w > 110 {
                Self::render_building(renderer, cx + 60 * s, ground_y, 5 * s, 15, &style, is_day)?;
                Self::render_antenna(renderer, cx + 62 * s, ground_y.saturating_sub(15), 4, style.antenna)?;
                Self::render_building(renderer, cx + 67 * s, ground_y, 4 * s, 7, &style, is_day)?;
            }

            // ── Left-side skyscraper wall ──
            let lbx = cx.saturating_sub(18 * s);
            Self::render_building(renderer, lbx, ground_y, 7 * s, 14, &style, is_day)?;
            Self::render_building(renderer, lbx.saturating_sub(10 * s), ground_y, 5 * s, 10, &style, is_day)?;
            Self::render_building(renderer, lbx.saturating_sub(17 * s), ground_y, 5 * s, 17, &style, is_day)?;
            Self::render_antenna(renderer, lbx.saturating_sub(14 * s), ground_y.saturating_sub(17), 3, style.antenna)?;
            // Extra-wide: more left-side towers
            if w > 90 {
                Self::render_building(renderer, lbx.saturating_sub(24 * s), ground_y, 4 * s, 8, &style, is_day)?;
                Self::render_building(renderer, lbx.saturating_sub(30 * s), ground_y, 4 * s, 12, &style, is_day)?;
            }
            if w > 110 {
                Self::render_building(renderer, lbx.saturating_sub(36 * s), ground_y, 5 * s, 11, &style, is_day)?;
                Self::render_building(renderer, lbx.saturating_sub(43 * s), ground_y, 4 * s, 6, &style, is_day)?;
            }

            // Statue of Liberty — far left, filled green with golden torch
            let lib_cx = lbx.saturating_sub(50 * s).max(6);
            let lib_lines: Vec<&str> = STATUE_LIBERTY_ART.lines().collect();
            let lib_w = lib_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
            let lib_h = lib_lines.len() as u16;
            let lib_sx = lib_cx.saturating_sub(lib_w / 2);
            let lib_sy = ground_y.saturating_sub(lib_h);
            // Green body fill — solid silhouette
            for row in 0..lib_h {
                for col in 0..lib_w {
                    if lib_lines[row as usize].chars().nth(col as usize) != Some(' ') {
                        renderer.render_char(lib_sx + col, lib_sy + row, '#', Color::Rgb { r: 0, g: 100, b: 20 })?;
                    }
                }
            }
            // Dark green background aura
            for row in 0..lib_h {
                renderer.render_char(lib_sx.saturating_sub(1), lib_sy + row, '│', Color::Rgb { r: 20, g: 70, b: 10 })?;
                renderer.render_char(lib_sx + lib_w, lib_sy + row, '│', Color::Rgb { r: 20, g: 70, b: 10 })?;
            }
            // Golden torch — flame at the top
            let torch_x = lib_cx + 2;
            let torch_y = lib_sy.saturating_sub(1);
            let flame_colors = [
                Color::Rgb { r: 255, g: 215, b: 0 },   // gold
                Color::Rgb { r: 255, g: 160, b: 0 },   // orange
                Color::Rgb { r: 255, g: 80, b: 0 },    // deep orange
                Color::Rgb { r: 255, g: 200, b: 0 },   // bright gold
            ];
            // Flame core
            renderer.render_char(torch_x, torch_y, '▲', flame_colors[0])?;
            renderer.render_char(torch_x, torch_y.saturating_sub(1), '♦', flame_colors[1])?;
            // Flame glow
            renderer.render_char(torch_x.saturating_sub(1), torch_y, '*', flame_colors[2])?;
            renderer.render_char(torch_x + 1, torch_y, '*', flame_colors[3])?;
            renderer.render_char(torch_x.saturating_sub(2), torch_y.saturating_sub(1), '.', flame_colors[0])?;
            renderer.render_char(torch_x + 2, torch_y.saturating_sub(1), '.', flame_colors[1])?;
            renderer.render_char(torch_x.saturating_sub(1), torch_y.saturating_sub(2), '°', flame_colors[3])?;
            renderer.render_char(torch_x + 1, torch_y.saturating_sub(2), '°', flame_colors[2])?;
            renderer.render_char(torch_x, torch_y.saturating_sub(3), '.', flame_colors[0])?;
            // Render landmark outline in bright green on top
            Self::render_landmark(renderer, STATUE_LIBERTY_ART, lib_cx, ground_y, Color::Rgb { r: 100, g: 255, b: 80 })?;

        } else if city.contains("london") {
            // ── London ── Big Ben + Tower Bridge + varied heights
            // Big Ben left of center so there's room for the modern skyline right
            Self::render_landmark(renderer, LONDON_ART, cx.saturating_sub(12 * s), ground_y, style.landmark)?;
            // Modern tall tower (Shard-style) right of center
            let shard_x = cx + 8 * s;
            Self::render_building(renderer, shard_x, ground_y, 7 * s, 20, &style, is_day)?;
            Self::render_antenna(renderer, shard_x + 3 * s, ground_y.saturating_sub(20), 4, style.antenna)?;
            // Medium building between Big Ben and Shard
            if cx > 6 * s {
                Self::render_building(renderer, cx.saturating_sub(4 * s), ground_y, 5 * s, 10, &style, is_day)?;
            }
            // Tower Bridge far right
            if w > 60 {
                Self::render_landmark(renderer, TOWER_BRIDGE_ART, (w as f32 * 0.82) as u16, ground_y, Color::Grey)?;
            }
            // Classic low buildings far left
            if cx > 24 * s {
                Self::render_building(renderer, cx.saturating_sub(24 * s), ground_y, 6 * s, 8, &style, is_day)?;
            }
            if w > 110 && cx > 32 * s {
                Self::render_building(renderer, cx.saturating_sub(32 * s), ground_y, 5 * s, 6, &style, is_day)?;
            }

        } else if city.contains("paris") {
            // ── Paris ── Angled Eiffel Tower + Arc de Triomphe + ornate cityscape
            // Eiffel Tower slightly left so Arc fits on the right
            let eiffel_x = cx.saturating_sub(3 * s);
            Self::render_landmark(renderer, PARIS_ART, eiffel_x, ground_y, style.landmark)?;

            // Arc de Triomphe — right side, warm stone
            let arc_x = cx + 22 * s;
            Self::render_landmark(renderer, ARC_ART, arc_x, ground_y, Color::DarkYellow)?;

            // ── Left-side Haussmann blocks — pushed clear of the tower ──
            Self::render_building(renderer, eiffel_x.saturating_sub(18 * s), ground_y, 7 * s, 13, &style, is_day)?;
            if eiffel_x > 28 * s {
                Self::render_building(renderer, eiffel_x.saturating_sub(28 * s), ground_y, 5 * s, 10, &style, is_day)?;
            }
            if w > 90 && eiffel_x > 36 * s {
                Self::render_building(renderer, eiffel_x.saturating_sub(36 * s), ground_y, 6 * s, 8, &style, is_day)?;
            }
            if w > 110 && eiffel_x > 44 * s {
                Self::render_building(renderer, eiffel_x.saturating_sub(44 * s), ground_y, 5 * s, 9, &style, is_day)?;
            }

            // ── Building between Eiffel and Arc — pushed right to clear both ──
            Self::render_building(renderer, cx + 13 * s, ground_y, 5 * s, 10, &style, is_day)?;

            // ── Right of Arc de Triomphe ──
            Self::render_building(renderer, arc_x + 10 * s, ground_y, 7 * s, 12, &style, is_day)?;
            if w > 90 {
                Self::render_building(renderer, arc_x + 19 * s, ground_y, 5 * s, 9, &style, is_day)?;
            }
            if w > 110 {
                Self::render_building(renderer, arc_x + 26 * s, ground_y, 6 * s, 14, &style, is_day)?;
                Self::render_antenna(renderer, arc_x + 29 * s, ground_y.saturating_sub(14), 3, Color::White)?;
            }

            // ── Roundabout fountain at the base ──
            let fy = ground_y;
            let fx = eiffel_x + 2;
            renderer.render_char(fx.saturating_sub(4), fy, '(', Color::Cyan)?;
            renderer.render_char(fx.saturating_sub(3), fy, '~', Color::Rgb { r: 100, g: 200, b: 255 })?;
            renderer.render_char(fx.saturating_sub(2), fy, '~', Color::Rgb { r: 100, g: 200, b: 255 })?;
            renderer.render_char(fx.saturating_sub(1), fy, 'o', Color::Rgb { r: 0, g: 220, b: 255 })?;
            renderer.render_char(fx, fy, '~', Color::Rgb { r: 100, g: 200, b: 255 })?;
            renderer.render_char(fx + 1, fy, '~', Color::Rgb { r: 100, g: 200, b: 255 })?;
            renderer.render_char(fx + 2, fy, ')', Color::Cyan)?;
            // Fountain spray
            renderer.render_char(fx.saturating_sub(1), fy.saturating_sub(1), '\'', Color::Rgb { r: 180, g: 230, b: 255 })?;
            renderer.render_char(fx, fy.saturating_sub(1), '\'', Color::Rgb { r: 180, g: 230, b: 255 })?;
            renderer.render_char(fx + 1, fy.saturating_sub(1), '\'', Color::Rgb { r: 180, g: 230, b: 255 })?;

        } else if city.contains("berlin") {
            // ── Berlin ── Brandenburg Gate + mix of eras
            Self::render_landmark(renderer, GATE_ART, cx, ground_y, Color::DarkYellow)?;
            Self::render_building(renderer, cx + 14 * s, ground_y, 9 * s, 14, &style, is_day)?;
            Self::render_building(renderer, cx + 25 * s, ground_y, 7 * s, 10, &style, is_day)?;
            if cx > 18 * s {
                Self::render_building(renderer, cx.saturating_sub(18 * s), ground_y, 8 * s, 11, &style, is_day)?;
            }
            if cx > 28 * s {
                Self::render_building(renderer, cx.saturating_sub(28 * s), ground_y, 6 * s, 8, &style, is_day)?;
            }

        } else if city.contains("sydney") {
            // ── Sydney ── Opera House + harbour city
            Self::render_landmark(renderer, OPERA_HOUSE_ART, cx, ground_y, Color::White)?;
            Self::render_building(renderer, cx + 14 * s, ground_y, 8 * s, 12, &style, is_day)?;
            Self::render_building(renderer, cx + 24 * s, ground_y, 6 * s, 9, &style, is_day)?;
            if cx > 18 * s {
                Self::render_building(renderer, cx.saturating_sub(18 * s), ground_y, 7 * s, 14, &style, is_day)?;
            }
            if cx > 27 * s {
                Self::render_building(renderer, cx.saturating_sub(27 * s), ground_y, 5 * s, 8, &style, is_day)?;
            }

        } else if city.contains("tokyo") {
            // ════════════════════════════════════════════════════
            //  TOKYO — Fuji · Temples · Skyscrapers · Neon Night
            // ════════════════════════════════════════════════════

            // ── Layer 0: Mount Fuji + skyline backdrop ──
            let fuji_lines: Vec<&str> = TOKYO_SKYLINE_ART.lines().collect();
            if !fuji_lines.is_empty() {
                let art_w = fuji_lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
                let art_h = fuji_lines.len() as u16;
                let start_y = ground_y.saturating_sub(art_h + 3);
                let offset_x = if w > art_w { (w - art_w) / 2 } else { 0 };
                for (row, line) in fuji_lines.iter().enumerate() {
                    let y = start_y + row as u16;
                    for (col, ch) in line.chars().enumerate() {
                        if ch == ' ' { continue; }
                        let x = offset_x + col as u16;
                        if x >= w { continue; }
                        let color = if row < 5 {
                            // Fuji peak — cool whites
                            if is_day { Color::Rgb { r: 220, g: 225, b: 240 } }
                            else { Color::Rgb { r: 80, g: 90, b: 120 } }
                        } else if row < 10 {
                            // Mid mountain — greys
                            if is_day { Color::Rgb { r: 160, g: 170, b: 190 } }
                            else { Color::Rgb { r: 90, g: 95, b: 125 } }
                        } else {
                            // City base — urban glow
                            if is_day { Color::Rgb { r: 120, g: 130, b: 155 } }
                            else { Color::Rgb { r: 110, g: 110, b: 135 } }
                        };
                        renderer.render_char(x, y, ch, color)?;
                    }
                }
            }

            // ── Layer 1: Pagoda (ornate multi-tier, far left) ──
            let pagoda_x = (w as f32 * 0.16) as u16;
            Self::render_landmark(renderer, PAGODA_ART, pagoda_x, ground_y, Color::DarkYellow)?;

            // ── Layer 2: Tokyo Tower (iconic red, right of center) ──
            Self::render_landmark(renderer, TOKYO_TOWER_ART, cx + 2 * s, ground_y, Color::Red)?;

            // ── Layer 3: Dense skyscraper canyon ──
            // Center cluster
            Self::render_building(renderer, cx + 9 * s, ground_y, 8 * s, 18, &style, is_day)?;
            Self::render_building(renderer, cx + 19 * s, ground_y, 6 * s, 14, &style, is_day)?;
            Self::render_building(renderer, cx + 27 * s, ground_y, 7 * s, 16, &style, is_day)?;
            // Left cluster
            if cx > 6 * s {
                Self::render_building(renderer, cx.saturating_sub(6 * s), ground_y, 7 * s, 13, &style, is_day)?;
            }
            if cx > 14 * s {
                Self::render_building(renderer, cx.saturating_sub(14 * s), ground_y, 5 * s, 9, &style, is_day)?;
            }
            // Right side
            Self::render_building(renderer, cx + 36 * s, ground_y, 6 * s, 11, &style, is_day)?;
            if w > 90 {
                Self::render_building(renderer, cx + 44 * s, ground_y, 7 * s, 13, &style, is_day)?;
                Self::render_building(renderer, cx + 53 * s, ground_y, 5 * s, 9, &style, is_day)?;
            }
            if w > 110 {
                Self::render_building(renderer, cx + 60 * s, ground_y, 6 * s, 14, &style, is_day)?;
                Self::render_building(renderer, cx + 68 * s, ground_y, 5 * s, 7, &style, is_day)?;
            }
            // Far left more buildings
            if cx > 22 * s {
                Self::render_building(renderer, cx.saturating_sub(22 * s), ground_y, 6 * s, 11, &style, is_day)?;
            }

            // ── Layer 4: Torii gate (ornate, far right) ──
            if w > 70 {
                let torii_x = (w as f32 * 0.88) as u16;
                Self::render_landmark(renderer, TORII_ART, torii_x.min(w.saturating_sub(16)), ground_y, Color::DarkRed)?;
            }

            // ── Layer 5: NEON EXTRAVAGANZA (night only) ──
            if !is_day {
                let ncolors = [
                    Color::Rgb { r: 255, g: 40, b: 90 },   // hot pink
                    Color::Rgb { r: 0, g: 255, b: 200 },   // cyan
                    Color::Rgb { r: 255, g: 220, b: 0 },   // yellow
                    Color::Rgb { r: 255, g: 80, b: 0 },    // orange
                    Color::Rgb { r: 180, g: 0, b: 255 },   // purple
                    Color::Rgb { r: 0, g: 255, b: 100 },   // green
                    Color::Rgb { r: 255, g: 255, b: 255 }, // white
                ];

                // Large neon billboards on buildings
                #[rustfmt::skip]
                let signs: &[(u16, u16, &str)] = &[
                    (cx + 12 * s, ground_y.saturating_sub(6),  "TOKYO"),
                    (cx + 24 * s, ground_y.saturating_sub(4),  "SHIBUYA"),
                    (cx.saturating_sub(4 * s), ground_y.saturating_sub(8),  "ラーメン"),
                    (cx + 38 * s, ground_y.saturating_sub(3),  "すし"),
                    (cx + 55 * s, ground_y.saturating_sub(7),  "OPEN"),
                    (cx + 17 * s, ground_y.saturating_sub(11), "BAR"),
                    (cx.saturating_sub(12 * s), ground_y.saturating_sub(4), "DONKI"),
                    (cx + 46 * s, ground_y.saturating_sub(9),  "パチンコ"),
                ];
                for &(sx, sy, text) in signs {
                    for (i, ch) in text.chars().enumerate() {
                        let nc = ncolors[i % ncolors.len()];
                        renderer.render_char(sx + i as u16, sy, ch, nc)?;
                        // Glow bloom
                        renderer.render_char(sx + i as u16, sy.saturating_sub(1), '·', nc)?;
                        renderer.render_char(sx + i as u16, sy.saturating_sub(2), '.', nc)?;
                    }
                }

                // Vertical neon strips on building edges
                for i in 0u16..(w / 5) {
                    let nx = i * 5 + 2;
                    if nx < w {
                        let nc = ncolors[(i as usize) % ncolors.len()];
                        for dy in 0..8u16 {
                            let ny = ground_y.saturating_sub(3 + dy);
                            let ch = if dy % 2 == 0 { '┃' } else { '│' };
                            renderer.render_char(nx, ny, ch, nc)?;
                        }
                    }
                }

                // Scattered neon pips on buildings
                for i in 0u16..(w / 3) {
                    let nx = i * 3 + 1;
                    if nx < w {
                        let nc = ncolors[((i as usize) * 3) % ncolors.len()];
                        let ny = ground_y.saturating_sub(5 + (i % 12));
                        renderer.render_char(nx, ny, '▪', nc)?;
                    }
                }

                // Ground-level neon: vending machines, street glow
                for i in 0u16..(w / 4) {
                    let gx = i * 4 + 1;
                    if gx < w {
                        let gc = ncolors[((i as usize + 2) * 5) % ncolors.len()];
                        renderer.render_char(gx, ground_y, '▂', gc)?;
                    }
                }
            }

            // ── Layer 6: Stone lanterns + cherry blossoms ──
            if is_day {
                // Lanterns (tōrō) at ground level — stone
                for &(lx, ly) in &[
                    (cx.saturating_sub(30 * s), ground_y),
                    (cx + 30 * s, ground_y),
                ] {
                    if lx > 2 && lx < w.saturating_sub(4) {
                        renderer.render_char(lx, ly, '◈', Color::Rgb { r: 160, g: 150, b: 130 })?;
                        renderer.render_char(lx, ly.saturating_sub(1), '▣', Color::Rgb { r: 140, g: 130, b: 110 })?;
                        renderer.render_char(lx, ly.saturating_sub(2), '▲', Color::DarkGrey)?;
                    }
                }

                // Cherry blossom trees
                for &(bx, by) in &[
                    (cx.saturating_sub(28 * s), ground_y.saturating_sub(5)),
                    (cx + 34 * s, ground_y.saturating_sub(4)),
                    (cx.saturating_sub(12 * s), ground_y.saturating_sub(6)),
                ] {
                    if bx > 2 && bx < w.saturating_sub(8) {
                        let pink = Color::Rgb { r: 255, g: 180, b: 200 };
                        renderer.render_char(bx.saturating_sub(2), by, '✿', pink)?;
                        renderer.render_char(bx.saturating_sub(1), by, '❀', pink)?;
                        renderer.render_char(bx, by, '✿', Color::Rgb { r: 255, g: 150, b: 180 })?;
                        renderer.render_char(bx + 1, by, '❀', pink)?;
                        renderer.render_char(bx + 2, by, '✿', pink)?;
                        renderer.render_char(bx.saturating_sub(1), by.saturating_sub(1), '✿', Color::Rgb { r: 255, g: 130, b: 170 })?;
                        renderer.render_char(bx, by.saturating_sub(1), '❀', pink)?;
                        renderer.render_char(bx + 1, by.saturating_sub(1), '✿', pink)?;
                    }
                }
            }

            // ── Layer 7: Night lantern glow ──
            if !is_day {
                for &(lx, ly) in &[
                    (cx.saturating_sub(30 * s), ground_y),
                    (cx + 30 * s, ground_y),
                    (pagoda_x.max(3), ground_y.saturating_sub(1)),
                ] {
                    if lx > 1 && lx < w.saturating_sub(3) {
                        renderer.render_char(lx, ly, '●', Color::Rgb { r: 255, g: 160, b: 40 })?;
                        renderer.render_char(lx.saturating_sub(1), ly.saturating_sub(1), '·', Color::Rgb { r: 255, g: 200, b: 80 })?;
                        renderer.render_char(lx + 1, ly.saturating_sub(1), '·', Color::Rgb { r: 255, g: 200, b: 80 })?;
                        renderer.render_char(lx, ly.saturating_sub(1), '°', Color::Rgb { r: 255, g: 220, b: 100 })?;
                        renderer.render_char(lx, ly.saturating_sub(2), '.', Color::Rgb { r: 255, g: 180, b: 60 })?;
                    }
                }

                // Moon visible at night over Fuji
                let moon_x = (w as f32 * 0.25) as u16;
                let moon_y = ground_y.saturating_sub(20).max(2);
                renderer.render_char(moon_x, moon_y, '◯', Color::Rgb { r: 255, g: 240, b: 200 })?;
                renderer.render_char(moon_x.saturating_sub(1), moon_y.saturating_sub(1), '·', Color::Rgb { r: 255, g: 255, b: 220 })?;
                renderer.render_char(moon_x + 1, moon_y.saturating_sub(1), '·', Color::Rgb { r: 255, g: 255, b: 220 })?;
            }

            // ── Layer 8: Ground street details ──
            for col in 1..w.saturating_sub(1) {
                if col % 28 < 14 {
                    renderer.render_char(col, ground_y, '─', Color::DarkGrey)?;
                }
            }
            // Crosswalk stripes near center
            for i in 0..6u16 {
                let cxw = cx.saturating_sub(5) + i * 2;
                for row in 0..3u16 {
                    renderer.render_char(cxw, ground_y.saturating_sub(row), '┃',
                        if is_day { Color::White } else { Color::Rgb { r: 0, g: 255, b: 100 } })?;
                }
            }

        } else if city.contains("dubai") {
            // ── Dubai ── Burj Khalifa + futuristic towers
            Self::render_landmark(renderer, BURJ_ART, cx, ground_y, style.landmark)?;
            Self::render_building(renderer, cx + 16 * s, ground_y, 8 * s, 12, &style, is_day)?;
            Self::render_building(renderer, cx + 26 * s, ground_y, 6 * s, 9, &style, is_day)?;
            if cx > 20 * s {
                Self::render_building(renderer, cx.saturating_sub(20 * s), ground_y, 8 * s, 10, &style, is_day)?;
            }
            if cx > 30 * s {
                Self::render_building(renderer, cx.saturating_sub(30 * s), ground_y, 6 * s, 8, &style, is_day)?;
            }

        } else if city.contains("rio") {
            // ── Rio ── Christ the Redeemer on hill + colorful city
            Self::render_landmark(renderer, CHRIST_ART, cx, ground_y, Color::White)?;
            Self::render_building(renderer, cx + 12 * s, ground_y, 8 * s, 13, &style, is_day)?;
            Self::render_building(renderer, cx + 22 * s, ground_y, 6 * s, 10, &style, is_day)?;
            if cx > 16 * s {
                Self::render_building(renderer, cx.saturating_sub(16 * s), ground_y, 7 * s, 11, &style, is_day)?;
            }
            if cx > 25 * s {
                Self::render_building(renderer, cx.saturating_sub(25 * s), ground_y, 5 * s, 8, &style, is_day)?;
            }

        } else if city.contains("chicago") {
            // ── Chicago ── Willis/Sears Tower + dense Loop
            let bw = 10 * s;
            let bh = 22 + s * 2;
            let bx = cx.saturating_sub(bw / 2);
            Self::render_building(renderer, bx, ground_y, bw, bh, &style, is_day)?;
            Self::render_antenna(renderer, cx, ground_y.saturating_sub(bh), 5, style.antenna)?;
            // John Hancock style right
            Self::render_building(renderer, cx + 14 * s, ground_y, 8 * s, 18, &style, is_day)?;
            Self::render_antenna(renderer, cx + 18 * s, ground_y.saturating_sub(18), 3, style.antenna)?;
            Self::render_building(renderer, cx + 25 * s, ground_y, 6 * s, 12, &style, is_day)?;
            if cx > 18 * s {
                Self::render_building(renderer, cx.saturating_sub(18 * s), ground_y, 7 * s, 14, &style, is_day)?;
            }
            if cx > 27 * s && w > 100 {
                Self::render_building(renderer, cx.saturating_sub(27 * s), ground_y, 6 * s, 10, &style, is_day)?;
            }

        } else if city.contains("amsterdam") {
            // ── Amsterdam ── narrow canal houses
            let hw = 6 * s;
            for i in 0i32..6 {
                let bh = 9 + ((i as u16 + 1) % 4) * 2;
                let x_off = (i - 2) * (hw as i32 + 2);
                let bx = (cx as i32 + x_off - hw as i32 / 2).max(0) as u16;
                if bx + hw < w {
                    Self::render_building(renderer, bx, ground_y, hw, bh, &style, is_day)?;
                }
            }

        } else if city.contains("miami") {
            // ── Miami ── Art Deco skyline with pastel towers
            // Centerpiece: tall stepped Art Deco tower
            let main_bw = 10 * s;
            let main_bh = 20;
            let main_bx = cx.saturating_sub(main_bw / 2);
            Self::render_building(renderer, main_bx, ground_y, main_bw, main_bh, &style, is_day)?;
            Self::render_antenna(renderer, cx, ground_y.saturating_sub(main_bh), 4, Color::Magenta)?;
            // Crown / decorative top
            let crown_y = ground_y.saturating_sub(main_bh);
            renderer.render_char(cx, crown_y.saturating_sub(1), '◆', Color::Rgb { r: 255, g: 120, b: 160 })?;

            // Right-side cluster
            Self::render_building(renderer, cx + 14 * s, ground_y, 8 * s, 14, &style, is_day)?;
            Self::render_building(renderer, cx + 24 * s, ground_y, 6 * s, 10, &style, is_day)?;
            Self::render_building(renderer, cx + 32 * s, ground_y, 5 * s, 7, &style, is_day)?;
            if w > 90 {
                Self::render_building(renderer, cx + 39 * s, ground_y, 7 * s, 12, &style, is_day)?;
                Self::render_antenna(renderer, cx + 42 * s, ground_y.saturating_sub(12), 2, Color::Rgb { r: 255, g: 180, b: 100 })?;
            }
            if w > 110 {
                Self::render_building(renderer, cx + 48 * s, ground_y, 5 * s, 8, &style, is_day)?;
                Self::render_building(renderer, cx + 55 * s, ground_y, 6 * s, 15, &style, is_day)?;
            }

            // Left-side cluster
            if cx > 16 * s {
                Self::render_building(renderer, cx.saturating_sub(16 * s), ground_y, 8 * s, 12, &style, is_day)?;
            }
            if cx > 26 * s {
                Self::render_building(renderer, cx.saturating_sub(26 * s), ground_y, 6 * s, 9, &style, is_day)?;
            }
            if w > 90 && cx > 34 * s {
                Self::render_building(renderer, cx.saturating_sub(34 * s), ground_y, 5 * s, 11, &style, is_day)?;
            }
            if w > 110 && cx > 41 * s {
                Self::render_building(renderer, cx.saturating_sub(41 * s), ground_y, 7 * s, 14, &style, is_day)?;
                Self::render_antenna(renderer, cx.saturating_sub(38 * s), ground_y.saturating_sub(14), 3, Color::Cyan)?;
            }

            // ── Waterfront edge ──
            for col in 2..(w.saturating_sub(2)) {
                if col % 6 < 3 {
                    renderer.render_char(col, ground_y, '~', Color::Cyan)?;
                }
            }

        } else {
            // ── Generic city ──
            let bw = 10 * s;
            let bh = 18 + s * 2;
            let bx = cx.saturating_sub(bw / 2);
            Self::render_building(renderer, bx, ground_y, bw, bh, &style, is_day)?;
            Self::render_antenna(renderer, cx, ground_y.saturating_sub(bh), 4, style.antenna)?;

            if w > 60 {
                let l_bx = cx.saturating_sub(8 * s / 2 + 16 * s);
                Self::render_building(renderer, l_bx, ground_y, 8 * s, 14 + s, &style, is_day)?;
                Self::render_building(renderer, cx + 14 * s, ground_y, 8 * s, 12, &style, is_day)?;
            }
            if w > 90 {
                let fl_bx = cx.saturating_sub(6 * s / 2 + 28 * s);
                Self::render_building(renderer, fl_bx, ground_y, 6 * s, 8, &style, is_day)?;
                let fr_bx = cx + 24 * s;
                Self::render_building(renderer, fr_bx, ground_y, 7 * s, 16, &style, is_day)?;
                Self::render_antenna(renderer, fr_bx + 3 * s, ground_y.saturating_sub(16), 3, style.antenna)?;
            }
        }

        Ok(())
    }
}
