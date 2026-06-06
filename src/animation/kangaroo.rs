use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::Rng;
use std::io;

// Kangaroo frames: standing and mid-hop
const KANGA_STAND: &[&str] = &[
    r" __  ",
    r"(oo) ",
    r"(  ) ",
    r" \/ \",
    r"  |  ",
];

const KANGA_HOP: &[&str] = &[
    r"  __ ",
    r" (oo)",
    r"  ()",
    r"  /\\",
    r" /  /",
];

struct Kangaroo {
    x: f32,
    hop_phase: f32,
}

pub struct KangarooSystem {
    kangaroos: Vec<Kangaroo>,
    width: u16,
    height: u16,
}

impl KangarooSystem {
    pub fn new(width: u16, height: u16) -> Self {
        let kangaroos = vec![
            Kangaroo { x: (width as f32 * 0.15), hop_phase: 0.0 },
            Kangaroo { x: (width as f32 * 0.45), hop_phase: 1.5 },
        ];
        Self { kangaroos, width, height }
    }

    fn is_sydney(ctx: &FrameContext<'_>) -> bool {
        ctx.state
            .city_name
            .as_deref()
            .map(|n| n.to_lowercase().contains("sydney"))
            .unwrap_or(false)
            && ctx.state.active_scene_id == "city"
    }

    fn render_kanga(
        renderer: &mut TerminalRenderer,
        frame: &[&str],
        x: u16,
        ground_y: u16,
        color: Color,
    ) -> io::Result<()> {
        let h = frame.len() as u16;
        let start_y = ground_y.saturating_sub(h);
        for (row, line) in frame.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let cx = x + col as u16;
                    let cy = start_y + row as u16;
                    renderer.render_char(cx, cy, ch, color)?;
                }
            }
        }
        Ok(())
    }
}

impl AnimationSystem for KangarooSystem {
    fn id(&self) -> &'static str {
        "kangaroo"
    }

    fn layer(&self) -> RenderLayer {
        RenderLayer::PostScene
    }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_sydney(ctx)
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
    }

    fn update(&mut self, _ctx: &FrameContext<'_>, _rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        for k in &mut self.kangaroos {
            k.hop_phase = (k.hop_phase + 0.08) % (std::f32::consts::TAU);
            // Move right slowly, wrap around
            k.x += 0.12;
            if k.x > self.width as f32 {
                k.x = -6.0;
            }
        }
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, ctx: &FrameContext<'_>) -> io::Result<()> {
        if !Self::is_sydney(ctx) {
            return Ok(());
        }

        let ground_y = ctx.horizon_y;

        for k in &self.kangaroos {
            let x = k.x.round() as i32;
            if x < 0 || x + 6 >= self.width as i32 { continue; }
            let x = x as u16;

            // Vertical bounce: kangaroo hops up/down
            let bounce = ((k.hop_phase.sin() * 2.0).max(0.0)) as u16;
            let draw_y = ground_y.saturating_sub(bounce);

            let frame = if k.hop_phase.sin() > 0.3 { KANGA_HOP } else { KANGA_STAND };
            Self::render_kanga(renderer, frame, x, draw_y, Color::DarkYellow)?;
        }

        Ok(())
    }
}
