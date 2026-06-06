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
            // Tall slab right
            Self::render_building(renderer, cx + 26 * s, ground_y, 6 * s, 14, &style, is_day)?;
            // Buildings left of Empire State
            let lbx = cx.saturating_sub(18 * s);
            Self::render_building(renderer, lbx, ground_y, 7 * s, 12, &style, is_day)?;
            // Statue of Liberty — far left, bright green
            let lib_cx = lbx.saturating_sub(10).max(5);
            Self::render_landmark(renderer, STATUE_LIBERTY_ART, lib_cx, ground_y, Color::Green)?;
            // Extra density on wide terminals
            if w > 110 {
                Self::render_building(renderer, cx + 34 * s, ground_y, 5 * s, 9, &style, is_day)?;
                if lbx > 12 * s {
                    Self::render_building(renderer, lbx.saturating_sub(10 * s), ground_y, 6 * s, 10, &style, is_day)?;
                }
            }

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
            // ── Paris ── Eiffel Tower center + uniform Haussmann blocks
            Self::render_landmark(renderer, PARIS_ART, cx, ground_y, style.landmark)?;
            // Haussmann buildings: same height, evenly spaced both sides
            let bh = 11u16;
            let bw = 7 * s;
            let gap = 12 * s;
            for i in 1u16..=3 {
                let rx = cx + i * gap;
                let lx = cx.saturating_sub(i * gap + bw);
                if rx + bw < w {
                    Self::render_building(renderer, rx, ground_y, bw, bh, &style, is_day)?;
                }
                if lx + bw < cx {
                    Self::render_building(renderer, lx, ground_y, bw, bh, &style, is_day)?;
                }
            }

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
            // ── Tokyo ── Tower (red) + pagoda left + torii gate right + dense modern blocks
            // Tokyo Tower slightly right of center
            Self::render_landmark(renderer, TOKYO_TOWER_ART, cx + 4 * s, ground_y, Color::Red)?;
            // Pagoda — traditional silhouette left
            Self::render_landmark(renderer, PAGODA_ART, cx.saturating_sub(16 * s), ground_y, Color::DarkYellow)?;
            // Dense modern buildings
            Self::render_building(renderer, cx + 14 * s, ground_y, 8 * s, 16, &style, is_day)?;
            Self::render_building(renderer, cx + 24 * s, ground_y, 6 * s, 12, &style, is_day)?;
            if cx > 10 * s {
                Self::render_building(renderer, cx.saturating_sub(8 * s), ground_y, 6 * s, 14, &style, is_day)?;
            }
            // Torii gate far right if room
            if w > 100 {
                Self::render_landmark(renderer, TORII_ART, (w as f32 * 0.88) as u16, ground_y, Color::DarkRed)?;
            }
            if w > 120 && cx > 28 * s {
                Self::render_building(renderer, cx.saturating_sub(28 * s), ground_y, 5 * s, 10, &style, is_day)?;
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
            // ── Miami ── Art Deco tower + waterfront
            let bw = 10 * s;
            let bh = 16;
            let bx = cx.saturating_sub(bw / 2);
            Self::render_building(renderer, bx, ground_y, bw, bh, &style, is_day)?;
            Self::render_antenna(renderer, cx, ground_y.saturating_sub(bh), 3, Color::Magenta)?;
            Self::render_building(renderer, cx + 14 * s, ground_y, 7 * s, 12, &style, is_day)?;
            Self::render_building(renderer, cx + 23 * s, ground_y, 5 * s, 8, &style, is_day)?;
            if cx > 16 * s {
                Self::render_building(renderer, cx.saturating_sub(16 * s), ground_y, 7 * s, 10, &style, is_day)?;
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
