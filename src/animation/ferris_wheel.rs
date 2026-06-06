use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::f32::consts::TAU;
use std::io;

pub struct FerrisWheelSystem {
    rotation: f32,
    width: u16,
    height: u16,
}

impl FerrisWheelSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self { rotation: 0.0, width, height }
    }

    fn is_chicago(ctx: &FrameContext<'_>) -> bool {
        ctx.state
            .city_name
            .as_deref()
            .map(|n| n.to_lowercase().contains("chicago"))
            .unwrap_or(false)
            && ctx.state.active_scene_id == "city"
    }

    fn wheel_pos(ground_y: u16, w: u16) -> (u16, u16) {
        let cx = (w as f32 * 0.76) as u16;
        let cy = ground_y.saturating_sub(11);
        (cx, cy)
    }
}

fn spoke_char(angle: f32) -> char {
    use std::f32::consts::PI;
    let a = angle.rem_euclid(PI);
    if a < PI / 8.0 || a >= 7.0 * PI / 8.0 {
        '-'
    } else if a < 3.0 * PI / 8.0 {
        '/'
    } else if a < 5.0 * PI / 8.0 {
        '|'
    } else {
        '\\'
    }
}

impl AnimationSystem for FerrisWheelSystem {
    fn id(&self) -> &'static str {
        "ferris_wheel"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::PostScene
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_chicago(ctx)
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
    }

    fn update(&mut self, _ctx: &FrameContext<'_>, _rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        self.rotation = (self.rotation + 0.06) % TAU;
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_chicago(ctx) {
            return Ok(());
        }

        let ground_y = ctx.horizon_y;
        let w = self.width;
        if w < 40 { return Ok(()); }

        let (cx, cy) = Self::wheel_pos(ground_y, w);
        let rx = 10u16; // horizontal radius in chars (2× vertical for aspect ratio)
        let ry = 5u16;  // vertical radius in rows

        // Support structure (A-frame legs)
        for i in 1..=4u16 {
            let leg_y = cy + ry + i;
            if leg_y >= ground_y { break; }
            renderer.render_char(cx.saturating_sub(i), leg_y, '\\', Color::DarkYellow)?;
            renderer.render_char(cx + i, leg_y, '/', Color::DarkYellow)?;
            renderer.render_char(cx, leg_y, '|', Color::DarkYellow)?;
        }

        // Draw 4 cross-spokes
        for s in 0..4u16 {
            let angle = self.rotation + (s as f32 * TAU / 4.0);
            let ex = (rx as f32 * angle.cos()) as i32;
            let ey = (ry as f32 * angle.sin()) as i32;
            let steps = (rx + ry) as i32;
            let ch = spoke_char(angle);
            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let px = (cx as f32 + t * ex as f32).round() as i32;
                let py = (cy as f32 + t * ey as f32).round() as i32;
                if px >= 0 && py >= 0 && (px as u16) < w && (py as u16) < ground_y {
                    renderer.render_char(px as u16, py as u16, ch, Color::Yellow)?;
                }
                // Opposite spoke
                let px2 = (cx as f32 - t * ex as f32).round() as i32;
                let py2 = (cy as f32 - t * ey as f32).round() as i32;
                if px2 >= 0 && py2 >= 0 && (px2 as u16) < w && (py2 as u16) < ground_y {
                    renderer.render_char(px2 as u16, py2 as u16, ch, Color::Yellow)?;
                }
            }
        }

        // Draw 8 gondolas at rim
        for g in 0..8u16 {
            let angle = self.rotation + (g as f32 * TAU / 8.0);
            let gx = (cx as f32 + rx as f32 * angle.cos()).round() as i32;
            let gy = (cy as f32 + ry as f32 * angle.sin()).round() as i32;
            if gx >= 0 && gy >= 0 && (gx as u16) < w && (gy as u16) < ground_y {
                renderer.render_char(gx as u16, gy as u16, 'o', Color::Cyan)?;
            }
        }

        // Hub at center
        renderer.render_char(cx, cy, 'O', Color::White)?;

        Ok(())
    }
}
