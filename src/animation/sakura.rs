use crate::animation::{AnimationSystem, FrameCommands, FrameContext, RenderLayer, TerminalSize};
use crate::render::TerminalRenderer;
use crossterm::style::Color;
use rand::{Rng, RngExt};
use std::io;

struct Petal {
    x: f32,
    y: f32,
    fall_speed: f32,
    sway_phase: f32,
    sway_amp: f32,
    sway_freq: f32,
    size: u8,  // 0 = tiny '·', 1 = small '❀', 2 = large '✿'
}

pub struct SakuraSystem {
    petals: Vec<Petal>,
    width: u16,
    height: u16,
}

impl SakuraSystem {
    pub fn new(width: u16, height: u16) -> Self {
        Self { petals: Vec::new(), width, height }
    }

    fn is_tokyo(ctx: &FrameContext<'_>) -> bool {
        ctx.state.active_scene_id == "city"
            && ctx.state.city_name.as_deref()
                .map(|n| n.to_lowercase().contains("tokyo"))
                .unwrap_or(false)
    }
}

impl AnimationSystem for SakuraSystem {
    fn id(&self) -> &'static str { "sakura" }
    fn layer(&self) -> RenderLayer { RenderLayer::Foreground }

    fn is_active(&self, ctx: &FrameContext<'_>) -> bool {
        Self::is_tokyo(ctx) && ctx.conditions.sun.is_day
            && !ctx.conditions.is_raining && !ctx.conditions.is_thunderstorm
    }

    fn on_resize(&mut self, size: TerminalSize) {
        self.width = size.width;
        self.height = size.height;
        self.petals.clear();
    }

    fn update(&mut self, ctx: &FrameContext<'_>, rng: &mut dyn Rng, _commands: &mut FrameCommands) {
        if !Self::is_tokyo(ctx) { self.petals.clear(); return; }

        // Spawn petals
        if self.petals.len() < 25 && rng.random::<f32>() < 0.25 {
            let x = rng.random::<f32>() * self.width as f32;
            let y = -(rng.random::<f32>() * 8.0);
            let size = if rng.random::<f32>() < 0.15 { 2u8 }
                else if rng.random::<f32>() < 0.4 { 1u8 }
                else { 0u8 };
            self.petals.push(Petal {
                x, y,
                fall_speed: 0.03 + rng.random::<f32>() * 0.06,
                sway_phase: rng.random::<f32>() * std::f32::consts::TAU,
                sway_amp: 0.3 + rng.random::<f32>() * 0.7,
                sway_freq: 0.02 + rng.random::<f32>() * 0.04,
                size,
            });
        }

        // Animate
        for p in &mut self.petals {
            p.y += p.fall_speed;
            p.sway_phase += p.sway_freq;
            p.x += p.sway_amp * p.sway_phase.sin();
        }

        // Cull off-screen
        let max_y = ctx.horizon_y as f32 + 4.0;
        self.petals.retain(|p| p.y < max_y && p.x >= -2.0 && p.x < self.width as f32 + 2.0);
    }

    fn render(&mut self, renderer: &mut TerminalRenderer, _ctx: &FrameContext<'_>) -> io::Result<()> {
        for p in &self.petals {
            if p.y < 0.0 { continue; }
            let x = p.x as u16;
            let y = p.y as u16;
            if x < self.width && y < self.height {
                let (ch, color) = match p.size {
                    2 => ('✿', Color::Rgb { r: 255, g: 140, b: 180 }),
                    1 => ('❀', Color::Rgb { r: 255, g: 170, b: 200 }),
                    _ => ('·', Color::Rgb { r: 255, g: 200, b: 220 }),
                };
                renderer.render_char(x, y, ch, color)?;
            }
        }
        Ok(())
    }
}
