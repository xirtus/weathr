use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

// Skier frames — downhill carving
const SKIER_A: &[&str] = &[
    r"  O  ",
    r" /|\ ",
    r" / \ ",
    r"/   \",
];

const SKIER_B: &[&str] = &[
    r" O   ",
    r"/|   ",
    r" |\  ",
    r"/  \ ",
];

const SKIER_C: &[&str] = &[
    r"   O ",
    r"   |\",
    r"  /| ",
    r" /  \",
];

const SNOWBOARD_A: &[&str] = &[
    r" O ",
    r"\|/",
    r"/ \",
    r"[=]",
];

const SNOWBOARD_B: &[&str] = &[
    r" O ",
    r"/|\",
    r" | ",
    r"[=]",
];

struct Rider {
    x: f32,
    y: f32,
    speed_x: f32,
    speed_y: f32,
    phase: f32,
    kind: u8, // 0=skier, 1=snowboarder
    color: Color,
}

pub struct SkierSystem {
    riders: Vec<Rider>,
    width: u16,
    height: u16,
}

impl SkierSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self { riders: Vec::new(), width, height }
    }

    fn is_colorado(ctx: &FrameContext<'_>) -> bool {
        ctx.state.active_scene_id == "mountain"
            && ctx.state.city_name.as_deref()
                .map(|n| n.to_lowercase().contains("denver") || n.to_lowercase().contains("colorado"))
                .unwrap_or(false)
    }
}

impl AnimationSystem for SkierSystem {
    fn id(&self) -> &'static str { "skier" }
    fn layer(&self) -> RenderLayer { RenderLayer::PostScene }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_colorado(ctx) && !ctx.conditions.is_thunderstorm
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
        self.riders.clear();
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        if !Self::is_colorado(ctx) { self.riders.clear(); return; }

        let w = self.width as f32;
        let snowline = ctx.horizon_y as f32 - 3.0;

        if self.riders.len() < 5 && rng.random::<f32>() < 0.03 {
            let start_x = rng.random::<f32>() * w;
            let kind = if rng.random::<bool>() { 0u8 } else { 1u8 };
            self.riders.push(Rider {
                x: start_x,
                y: snowline - rng.random::<f32>() * 8.0,
                speed_x: 0.08 + rng.random::<f32>() * 0.2,
                speed_y: 0.03 + rng.random::<f32>() * 0.07,
                phase: rng.random::<f32>() * std::f32::consts::TAU,
                kind,
                color: match rng.random_range(0u8..4) {
                    0 => Color::Rgb { r: 255, g: 60, b: 60 },
                    1 => Color::Rgb { r: 60, g: 120, b: 255 },
                    2 => Color::Rgb { r: 255, g: 200, b: 0 },
                    _ => Color::Rgb { r: 40, g: 200, b: 80 },
                },
            });
        }

        for r in &mut self.riders {
            r.x += r.speed_x;
            r.y += r.speed_y;
            r.phase += 0.12;
        }

        self.riders.retain(|r| r.x < w + 6.0 && r.y < ctx.horizon_y as f32 + 4.0);
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_colorado(ctx) { return Ok(()); }

        for r in &self.riders {
            let x = r.x.round() as i32;
            let y = r.y.round() as i32;
            if x < -6 || x > self.width as i32 || y < 0 || y > self.height as i32 { continue; }
            let x = x.clamp(0, self.width as i32 - 1) as u16;
            let y = y.clamp(0, self.height as i32 - 1) as u16;

            let (frames, body_color) = if r.kind == 0 {
                (&[SKIER_A, SKIER_B, SKIER_C][..], Color::Rgb { r: 30, g: 40, b: 80 })
            } else {
                (&[SNOWBOARD_A, SNOWBOARD_B][..], Color::Rgb { r: 40, g: 30, b: 80 })
            };

            let fidx = (r.phase.sin() * frames.len() as f32 + frames.len() as f32) as usize % frames.len();
            let frame = frames[fidx];
            let h = frame.len() as u16;

            for (row, line) in frame.iter().enumerate() {
                for (col, ch) in line.chars().enumerate() {
                    if ch == ' ' { continue; }
                    let cy = y.saturating_add(row as u16);
                    let cx = x.saturating_add(col as u16);
                    let c = match ch {
                        'O' | 'o' => Color::Rgb { r: 255, g: 210, b: 150 },
                        '/' | '\\' | '|' => body_color,
                        '[' | ']' | '=' => r.color,
                        _ => body_color,
                    };
                    renderer.render_char(cx, cy, ch, c)?;
                }
            }
        }

        Ok(())
    }
}
