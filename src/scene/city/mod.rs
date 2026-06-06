mod style;

use crate::render::TerminalRenderer;
use crate::scene::{ChimneyPosition, Scene, SceneContext, SceneLayout};
use crossterm::style::Color;
use std::io;
use style::CitySceneStyle;

const NYC_ART: &str = include_str!("assets/nyc.txt");
const LONDON_ART: &str = include_str!("assets/london.txt");
const PARIS_ART: &str = include_str!("assets/paris.txt");
const GATE_ART: &str = include_str!("assets/gate.txt");
const OPERA_HOUSE_ART: &str = include_str!("assets/opera_house.txt");
const BURJ_ART: &str = include_str!("assets/burj.txt");
const TOKYO_TOWER_ART: &str = include_str!("assets/tokyo_tower.txt");
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
        let center_x = w / 2;
        let scale = (w / 80).max(1);

        Self::render_ground(renderer, w, ground_y, &style)?;

        let city = ctx
            .city_name
            .or(self.city_name.as_deref())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        if city.contains("new york") || city.contains("nyc") {
            // Empire State center + Statue of Liberty on the left
            Self::render_landmark(renderer, NYC_ART, center_x, ground_y, style.landmark)?;
            if w > 50 {
                let lib_x = center_x / 3;
                Self::render_landmark(renderer, STATUE_LIBERTY_ART, lib_x, ground_y, Color::DarkGreen)?;
            }
            Self::render_building(renderer, center_x + 14 * scale, ground_y, 8 * scale, 12, &style, is_day)?;
            if w > 80 && center_x > 22 * scale {
                Self::render_building(renderer, center_x.saturating_sub(22 * scale), ground_y, 7 * scale, 10, &style, is_day)?;
            }

        } else if city.contains("london") {
            // Big Ben center + Tower Bridge on the right
            Self::render_landmark(renderer, LONDON_ART, center_x, ground_y, style.landmark)?;
            if w > 60 {
                let bridge_x = center_x + center_x / 2;
                Self::render_landmark(renderer, TOWER_BRIDGE_ART, bridge_x, ground_y, Color::Grey)?;
            }
            if center_x > 16 * scale {
                Self::render_building(renderer, center_x.saturating_sub(16 * scale), ground_y, 6 * scale, 9, &style, is_day)?;
            }

        } else if city.contains("paris") {
            Self::render_landmark(renderer, PARIS_ART, center_x, ground_y, style.landmark)?;
            Self::render_building(renderer, center_x + 16 * scale, ground_y, 8 * scale, 10, &style, is_day)?;
            if center_x > 18 * scale {
                Self::render_building(renderer, center_x.saturating_sub(18 * scale), ground_y, 6 * scale, 8, &style, is_day)?;
            }

        } else if city.contains("berlin") {
            // Brandenburg Gate center + Reichstag-style building right
            Self::render_landmark(renderer, GATE_ART, center_x, ground_y, Color::DarkYellow)?;
            Self::render_building(renderer, center_x + 16 * scale, ground_y, 10 * scale, 12, &style, is_day)?;
            if center_x > 20 * scale {
                Self::render_building(renderer, center_x.saturating_sub(20 * scale), ground_y, 8 * scale, 10, &style, is_day)?;
            }

        } else if city.contains("sydney") {
            // Sydney Opera House center
            Self::render_landmark(renderer, OPERA_HOUSE_ART, center_x, ground_y, Color::White)?;
            Self::render_building(renderer, center_x + 16 * scale, ground_y, 8 * scale, 10, &style, is_day)?;
            if center_x > 20 * scale {
                Self::render_building(renderer, center_x.saturating_sub(20 * scale), ground_y, 7 * scale, 12, &style, is_day)?;
            }

        } else if city.contains("tokyo") {
            // Tokyo Tower in red center
            Self::render_landmark(renderer, TOKYO_TOWER_ART, center_x, ground_y, Color::Red)?;
            Self::render_building(renderer, center_x + 14 * scale, ground_y, 8 * scale, 14, &style, is_day)?;
            if center_x > 16 * scale {
                Self::render_building(renderer, center_x.saturating_sub(16 * scale), ground_y, 8 * scale, 12, &style, is_day)?;
            }

        } else if city.contains("dubai") {
            // Burj Khalifa — very tall spire
            Self::render_landmark(renderer, BURJ_ART, center_x, ground_y, style.landmark)?;
            Self::render_building(renderer, center_x + 18 * scale, ground_y, 9 * scale, 10, &style, is_day)?;
            if center_x > 22 * scale {
                Self::render_building(renderer, center_x.saturating_sub(22 * scale), ground_y, 8 * scale, 8, &style, is_day)?;
            }

        } else if city.contains("rio") {
            // Christ the Redeemer on a hill
            Self::render_landmark(renderer, CHRIST_ART, center_x, ground_y, Color::White)?;
            Self::render_building(renderer, center_x + 14 * scale, ground_y, 8 * scale, 12, &style, is_day)?;
            if center_x > 16 * scale {
                Self::render_building(renderer, center_x.saturating_sub(18 * scale), ground_y, 7 * scale, 10, &style, is_day)?;
            }

        } else if city.contains("chicago") {
            // Sears/Willis Tower style — very tall, ferris wheel via animation on right
            let bw = 10 * scale;
            let bh = 22 + scale * 2;
            let bx = center_x.saturating_sub(bw / 2);
            Self::render_building(renderer, bx, ground_y, bw, bh, &style, is_day)?;
            Self::render_antenna(renderer, center_x, ground_y.saturating_sub(bh), 5, style.antenna)?;
            if w > 60 {
                Self::render_building(renderer, center_x + 14 * scale, ground_y, 8 * scale, 14, &style, is_day)?;
            }
            if center_x > 20 * scale && w > 80 {
                Self::render_building(renderer, center_x.saturating_sub(20 * scale), ground_y, 7 * scale, 10, &style, is_day)?;
            }

        } else if city.contains("amsterdam") {
            // Narrow canal houses with stepped gable rooflines
            let house_w = 6 * scale;
            for i in 0i32..5 {
                let bh = 10 + ((i as u16 + 1) % 3) * 2;
                let x_off = (i - 2) * (house_w as i32 + 2);
                let bx = (center_x as i32 + x_off - house_w as i32 / 2).max(0) as u16;
                if bx + house_w < w {
                    Self::render_building(renderer, bx, ground_y, house_w, bh, &style, is_day)?;
                }
            }

        } else if city.contains("miami") {
            // Art Deco pastel tower
            let bw = 10 * scale;
            let bh = 16;
            let bx = center_x.saturating_sub(bw / 2);
            Self::render_building(renderer, bx, ground_y, bw, bh, &style, is_day)?;
            Self::render_antenna(renderer, center_x, ground_y.saturating_sub(bh), 3, Color::Magenta)?;
            if w > 60 {
                Self::render_building(renderer, center_x + 14, ground_y, 8, 12, &style, is_day)?;
                if center_x > 16 {
                    Self::render_building(renderer, center_x.saturating_sub(16), ground_y, 7, 10, &style, is_day)?;
                }
            }

        } else {
            // Generic city
            let bw = 10 * scale;
            let bh = 18 + scale * 2;
            let bx = center_x.saturating_sub(bw / 2);
            Self::render_building(renderer, bx, ground_y, bw, bh, &style, is_day)?;
            Self::render_antenna(renderer, center_x, ground_y.saturating_sub(bh), 4, style.antenna)?;

            if w > 60 {
                let l_bw = 8 * scale;
                let l_bh = 14 + scale;
                let l_bx = center_x.saturating_sub(l_bw / 2 + 16 * scale);
                Self::render_building(renderer, l_bx, ground_y, l_bw, l_bh, &style, is_day)?;

                let r_bw = 8 * scale;
                let r_bh = 12;
                let r_bx = center_x + 14 * scale;
                Self::render_building(renderer, r_bx, ground_y, r_bw, r_bh, &style, is_day)?;
            }

            if w > 90 {
                let fl_bw = 6 * scale;
                let fl_bx = center_x.saturating_sub(fl_bw / 2 + 28 * scale);
                Self::render_building(renderer, fl_bx, ground_y, fl_bw, 8, &style, is_day)?;

                let fr_bw = 7 * scale;
                let fr_bx = center_x + 24 * scale;
                Self::render_building(renderer, fr_bx, ground_y, fr_bw, 16, &style, is_day)?;
                Self::render_antenna(renderer, fr_bx + fr_bw / 2, ground_y.saturating_sub(16), 3, style.antenna)?;
            }
        }

        Ok(())
    }
}
