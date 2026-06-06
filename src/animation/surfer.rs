use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::io;

// Surfer on a wave: two frames
const SURFER_A: &[&str] = &[
    r" o  ",
    r"/|\ ",
    r"_|_ ",
    r"===~",
];

const SURFER_B: &[&str] = &[
    r"  o ",
    r" /| ",
    r"_/~ ",
    r"===~",
];

struct Surfer {
    x: f32,
    frame_phase: f32,
    speed: f32,
}

pub struct SurferSystem {
    surfers: Vec<Surfer>,
    width: u16,
    height: u16,
}

impl SurferSystem {
    pub fn new(width: u16, height: u16) -> Self {
        let surfers = vec![
            Surfer { x: width as f32 * 0.6, frame_phase: 0.0, speed: 0.3 },
            Surfer { x: width as f32 * 0.25, frame_phase: 1.2, speed: 0.45 },
        ];
        Self { surfers, width, height }
    }

    fn is_hawaii(ctx: &FrameContext<'_>) -> bool {
        let city = ctx
            .state
            .city_name
            .as_deref()
            .map(|n| n.to_lowercase())
            .unwrap_or_default();
        (city.contains("honolulu") || city.contains("hawaii"))
            && (ctx.state.active_scene_id == "beach" || ctx.state.active_scene_id == "santa_cruz")
    }

    fn render_surfer(
        renderer: &mut TerminalRenderer,
        frame: &[&str],
        x: u16,
        ground_y: u16,
    ) -> io::Result<()> {
        let h = frame.len() as u16;
        let start_y = ground_y.saturating_sub(h + 1);
        for (row, line) in frame.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch == ' ' { continue; }
                let color = match ch {
                    'o' => Color::Yellow,
                    '~' | '=' => Color::Cyan,
                    _ => Color::White,
                };
                renderer.render_char(x + col as u16, start_y + row as u16, ch, color)?;
            }
        }
        Ok(())
    }
}

impl AnimationSystem for SurferSystem {
    fn id(&self) -> &'static str {
        "surfer"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::Foreground
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_hawaii(ctx)
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
    }

    fn update(&mut self, _ctx: &FrameContext<'_>, _rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        for s in &mut self.surfers {
            s.frame_phase = (s.frame_phase + 0.1) % (std::f32::consts::TAU);
            // Surfers ride the wave rightward, then reset left
            s.x -= s.speed;
            if s.x < -5.0 {
                s.x = self.width as f32;
            }
        }
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_hawaii(ctx) {
            return Ok(());
        }

        let ground_y = ctx.horizon_y;

        for s in &self.surfers {
            let x = s.x.round() as i32;
            if x < 0 || x + 5 >= self.width as i32 { continue; }
            let x = x as u16;

            let frame = if s.frame_phase.sin() > 0.0 { SURFER_A } else { SURFER_B };
            Self::render_surfer(renderer, frame, x, ground_y)?;
        }

        Ok(())
    }
}
