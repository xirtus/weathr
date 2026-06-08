use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

/// Simplified person silhouette (3x3) for dense crowds
const PERSON_A: &[&str] = &[" o ", "/|\\", "/ \\"];
const PERSON_B: &[&str] = &[" o ", "/|\\", "_/ \\"];
const PERSON_C: &[&str] = &[" o ", "/| ", "/ \\"];

struct Walker {
    x: f32,
    y: u16,
    frame_phase: f32,
    speed: f32,
    direction: i8, // 1=right, -1=left
    color: Color,
}

pub struct ShibuyaSystem {
    walkers: Vec<Walker>,
    neon_tick: u32,
    width: u16,
    height: u16,
}

impl ShibuyaSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self { walkers: Vec::new(), neon_tick: 0, width, height }
    }

    fn is_tokyo(ctx: &FrameContext<'_>) -> bool {
        ctx.state.active_scene_id == "city"
            && ctx.state.city_name.as_deref()
                .map(|n| n.to_lowercase().contains("tokyo"))
                .unwrap_or(false)
    }
}

impl AnimationSystem for ShibuyaSystem {
    fn id(&self) -> &'static str { "shibuya" }
    fn layer(&self) -> RenderLayer { RenderLayer::PostScene }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_tokyo(ctx) && !ctx.conditions.is_thunderstorm
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
        self.walkers.clear();
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        if !Self::is_tokyo(ctx) { self.walkers.clear(); return; }

        self.neon_tick = self.neon_tick.wrapping_add(1);

        // Spawn walkers in crossing zone (center of screen, at ground level)
        if self.walkers.len() < 35 && rng.random::<f32>() < 0.4 {
            let cx = self.width as f32 / 2.0;
            let going_right = rng.random::<bool>();
            let start_x = if going_right {
                cx - rng.random::<f32>() * 20.0
            } else {
                cx + rng.random::<f32>() * 20.0
            };
            let color = match rng.random_range(0u8..5) {
                0 => Color::Rgb { r: 255, g: 200, b: 100 }, // warm
                1 => Color::Rgb { r: 100, g: 200, b: 255 }, // cool
                2 => Color::Rgb { r: 200, g: 100, b: 200 }, // purple
                3 => Color::Rgb { r: 255, g: 255, b: 255 }, // white
                _ => Color::Rgb { r: 150, g: 150, b: 150 }, // grey
            };
            self.walkers.push(Walker {
                x: start_x,
                y: ctx.horizon_y,
                frame_phase: rng.random::<f32>() * std::f32::consts::TAU,
                speed: 0.1 + rng.random::<f32>() * 0.3,
                direction: if going_right { 1 } else { -1 },
                color,
            });
        }

        // Animate
        for w in &mut self.walkers {
            w.x += w.speed * w.direction as f32;
            w.frame_phase += 0.15;
        }

        // Cull off-screen
        let cx = self.width as f32 / 2.0;
        self.walkers.retain(|w| {
            (w.x - cx).abs() < 35.0
        });
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_tokyo(ctx) { return Ok(()); }

        let ground_y = ctx.horizon_y;
        let is_day = ctx.conditions.sun.is_day;
        let crowd_color = if is_day {
            Color::Rgb { r: 60, g: 70, b: 80 }
        } else {
            Color::Rgb { r: 200, g: 190, b: 180 }
        };

        for w in &self.walkers {
            let x = w.x.round() as i32;
            if x < -3 || x > self.width as i32 + 3 { continue; }
            let x = x.clamp(0, self.width as i32 - 1) as u16;

            let frame_idx = (w.frame_phase.sin() * 3.0 + 3.0) as usize % 3;
            let frame = [PERSON_A, PERSON_B, PERSON_C][frame_idx];
            let h = frame.len() as u16;
            let start_y = ground_y.saturating_sub(h);

            for (row, line) in frame.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let draw_col = if w.direction == -1 {
                        x + (2 - col) as u16
                    } else {
                        x + col as u16
                    };
                    let c = if ch == 'o' { w.color } else { crowd_color };
                    renderer.render_char(draw_col, start_y + row as u16, ch, c)?;
                }
            }
        }

        // ── Neon crosswalk stripes (blinking, center of screen) ──
        if !is_day {
            let cx = self.width / 2;
            let stripe_on = (self.neon_tick / 30) % 2 == 0;
            if stripe_on {
                let stripe_color = Color::Rgb { r: 0, g: 255, b: 100 };
                for i in 0..8u16 {
                    let sx = cx.saturating_sub(16) + i * 4;
                    if sx < self.width {
                        for dx in 0..3u16 {
                            renderer.render_char(sx + dx, ground_y, '▬', stripe_color)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
